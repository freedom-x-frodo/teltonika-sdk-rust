use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use teltonika_core::config::ConnConfig;
use teltonika_core::{Result, TeltonikaError};

use crate::auth::{AuthCredentials, AuthState, AuthType};
use crate::error::from_reqwest;
use crate::utils::base64_encode;

#[derive(Deserialize)]
struct LoginResponse {
    data: LoginData,
}

#[derive(Deserialize)]
struct LoginData {
    token: String,
    expires: Option<u64>, // TODO: use for re-auth scheduling
}

#[derive(Clone)]
pub struct RestClient {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    http: reqwest::Client,
    base_url: String,
    auth: AuthState,
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

        let auth = match auth_type {
            AuthType::Session => {
                let credentials = AuthCredentials {
                    username: config.username,
                    password: config.password,
                };

                let response = http
                    .post(format!("{base_url}/api/login"))
                    .json(&credentials)
                    .send()
                    .await
                    .map_err(from_reqwest)?;

                if response.status() == 401 || response.status() == 403 {
                    return Err(TeltonikaError::AuthFailed {
                        username: credentials.username,
                    });
                }
                let response = response.error_for_status().map_err(from_reqwest)?;

                let token = response
                    .json::<LoginResponse>()
                    .await
                    .map_err(from_reqwest)?
                    .data
                    .token;

                AuthState::Session { token }
            }
            AuthType::Basic => {
                let encoded =
                    base64_encode(&format!("{}:{}", config.username, config.password));
                AuthState::Basic { encoded }
            }
        };

        Ok(Self {
            inner: Arc::new(ClientInner { http, base_url, auth }),
        })
    }

    fn auth_header(&self) -> String {
        match &self.inner.auth {
            AuthState::Session { token } => format!("Bearer {token}"),
            AuthState::Basic { encoded } => format!("Basic {encoded}"),
        }
    }
}