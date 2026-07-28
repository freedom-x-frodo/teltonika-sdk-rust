use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use teltonika_core::config::ConnConfig;
use teltonika_core::{Result, TeltonikaError};

use crate::auth::{AuthCredentials, AuthHeader, AuthType, login};
use crate::error::from_reqwest;
use crate::response::Envelope;

fn need_auth(status: reqwest::StatusCode) -> bool {
    status == 401 || status == 403
}

#[derive(Clone)]
pub struct RestClient {
    inner: Arc<ClientInner>,
}

impl std::fmt::Debug for RestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestClient")
            .field("base_url", &self.inner.base_url)
            .field("refresh_margin", &self.inner.refresh_margin)
            .finish_non_exhaustive()
    }
}

struct ClientInner {
    http: reqwest::Client,
    base_url: String,
    auth: RwLock<AuthHeader>,
    refresh_margin: Duration,
}

impl RestClient {
    pub async fn connect(config: ConnConfig, auth_type: AuthType) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()
            .map_err(from_reqwest)?;

        let base_url = config.endpoint;
        let refresh_margin = config.refresh_margin;

        let auth = match auth_type {
            AuthType::Session => {
                let credentials = AuthCredentials {
                    username: config.username,
                    password: config.password,
                };
                let login_data = login(&http, &base_url, &credentials).await?;
                AuthHeader::session(login_data, credentials)
            }
            AuthType::Basic => AuthHeader::basic(config.username, config.password),
        };

        Ok(Self {
            inner: Arc::new(ClientInner {
                http,
                base_url,
                auth: RwLock::new(auth),
                refresh_margin,
            }),
        })
    }

    async fn refresh_if_stale(&self) -> Result<()> {
        if !self
            .inner
            .auth
            .read()
            .await
            .is_stale(self.inner.refresh_margin)
        {
            return Ok(());
        }

        let mut auth = self.inner.auth.write().await;
        if !auth.is_stale(self.inner.refresh_margin) {
            return Ok(());
        }
        auth.refresh(&self.inner.http, &self.inner.base_url).await
    }

    async fn send_authorized_get(&self, path: &str) -> Result<reqwest::Response> {
        let header = self.inner.auth.read().await.value();

        self.inner
            .http
            .get(format!("{}/api{path}", self.inner.base_url))
            .header(reqwest::header::AUTHORIZATION, header)
            .send()
            .await
            .map_err(from_reqwest)
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.refresh_if_stale().await?;

        let mut response = self.send_authorized_get(path).await?;

        // Reactive path: the token died early, or the margin didn't cover it.
        if need_auth(response.status()) {
            let (can_refresh, username) = {
                let auth = self.inner.auth.read().await;
                (auth.can_refresh(), auth.username().to_string())
            };

            if !can_refresh {
                return Err(TeltonikaError::AuthFailed { username });
            }

            self.inner
                .auth
                .write()
                .await
                .refresh(&self.inner.http, &self.inner.base_url)
                .await?;

            response = self.send_authorized_get(path).await?;
            if need_auth(response.status()) {
                return Err(TeltonikaError::AuthFailed { username });
            }
        }

        let response = response.error_for_status().map_err(from_reqwest)?;

        response
            .json::<Envelope<T>>()
            .await
            .map_err(from_reqwest)?
            .into_data(&format!("GET {path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc as StdArc;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn login_body(token: &str, expires: u64) -> serde_json::Value {
        serde_json::json!({"success": true, "data": {"token": token, "expires": expires}})
    }

    fn data_body() -> serde_json::Value {
        serde_json::json!({"success": true, "data": {"value": 1}})
    }

    #[derive(serde::Deserialize, Debug)]
    struct Probe {
        value: u32,
    }

    async fn config_for(server: &MockServer) -> ConnConfig {
        ConnConfig::new("admin".into(), "pw".into(), server.uri())
    }

    #[tokio::test]
    async fn login_401_reports_username() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let err = RestClient::connect(config_for(&server).await, AuthType::Session)
            .await
            .unwrap_err();
        assert!(matches!(err, TeltonikaError::AuthFailed { username } if username == "admin"));
    }

    #[tokio::test]
    async fn basic_auth_never_logs_in() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .and(header("authorization", "Basic YWRtaW46cHc="))
            .respond_with(ResponseTemplate::new(200).set_body_json(data_body()))
            .mount(&server)
            .await;

        let client = RestClient::connect(config_for(&server).await, AuthType::Basic)
            .await
            .unwrap();
        let out: Probe = client.get("/probe").await.unwrap();
        assert_eq!(out.value, 1);
    }

    #[tokio::test]
    async fn basic_auth_401_fails_without_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let client = RestClient::connect(config_for(&server).await, AuthType::Basic)
            .await
            .unwrap();
        let err = client.get::<Probe>("/probe").await.unwrap_err();
        assert!(matches!(err, TeltonikaError::AuthFailed { .. }));
    }

    #[tokio::test]
    async fn reactive_401_refreshes_and_retries_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t1", 300)))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t2", 300)))
            .expect(1)
            .mount(&server)
            .await;

        // First GET rejects the stale token, second accepts the fresh one.
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .and(header("authorization", "Bearer t1"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .and(header("authorization", "Bearer t2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(data_body()))
            .expect(1)
            .mount(&server)
            .await;

        let client = RestClient::connect(config_for(&server).await, AuthType::Session)
            .await
            .unwrap();
        let out: Probe = client.get("/probe").await.unwrap();
        assert_eq!(out.value, 1);
    }

    #[tokio::test]
    async fn persistent_401_gives_up_after_one_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t", 300)))
            .expect(2) // connect + one refresh
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .respond_with(ResponseTemplate::new(401))
            .expect(2) // original + retry, no loop
            .mount(&server)
            .await;

        let client = RestClient::connect(config_for(&server).await, AuthType::Session)
            .await
            .unwrap();
        assert!(matches!(
            client.get::<Probe>("/probe").await.unwrap_err(),
            TeltonikaError::AuthFailed { .. }
        ));
    }

    #[tokio::test]
    async fn proactive_refresh_fires_before_request() {
        let server = MockServer::start().await;
        // expires: 0 => stale the instant connect returns.
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t", 0)))
            .expect(2) // connect, then proactive refresh on first get
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .respond_with(ResponseTemplate::new(200).set_body_json(data_body()))
            .expect(1)
            .mount(&server)
            .await;

        let client = RestClient::connect(config_for(&server).await, AuthType::Session)
            .await
            .unwrap();
        let _: Probe = client.get("/probe").await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_proactive_refresh_logs_in_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t", 0)))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        // Second and later logins return a long-lived token; if the
        // double-checked lock is broken this mock gets hit and the count fails.
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t2", 600)))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .respond_with(ResponseTemplate::new(200).set_body_json(data_body()))
            .mount(&server)
            .await;

        let client = StdArc::new(
            RestClient::connect(config_for(&server).await, AuthType::Session)
                .await
                .unwrap(),
        );
        let mut handles = Vec::new();
        for _ in 0..8 {
            let c = client.clone();
            handles.push(tokio::spawn(async move { c.get::<Probe>("/probe").await }));
        }
        for h in handles {
            h.await.unwrap().unwrap();
        }
    }
    #[tokio::test]
    async fn margin_larger_than_ttl_refreshes_once_per_request() {
        let server = MockServer::start().await;

        // 5min TTL against a 20min margin.
        Mock::given(method("POST"))
            .and(path("/api/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_body("t", 300)))
            .expect(4) // connect + one per get, not more
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/probe"))
            .respond_with(ResponseTemplate::new(200).set_body_json(data_body()))
            .expect(3)
            .mount(&server)
            .await;

        let config = ConnConfig::new("admin".into(), "pw".into(), server.uri())
            .with_refresh_margin(Duration::from_secs(20 * 60));

        let client = RestClient::connect(config, AuthType::Session)
            .await
            .unwrap();
        for _ in 0..3 {
            let _: Probe = client.get("/probe").await.unwrap();
        }
    }
}
