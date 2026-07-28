use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use teltonika_core::config::ConnConfig;
use teltonika_core::{Result, TeltonikaError};

use crate::auth::{login, AuthCredentials, AuthHeader, AuthType};
use crate::error::from_reqwest;
use crate::response::Envelope;

fn need_auth(status: reqwest::StatusCode) -> bool {
    status == 401 || status == 403
}

#[derive(Clone)]
pub struct RestClient {
    inner: Arc<ClientInner>,
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

        let base_url = format!("https://{}", config.ip);
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
        if !self.inner.auth.read().await.is_stale(self.inner.refresh_margin) {
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