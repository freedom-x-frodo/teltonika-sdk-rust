use std::str::FromStr;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use teltonika_core::{Result, TeltonikaError};

use crate::error::from_reqwest;
use crate::response::Envelope;
use crate::utils::base64_encode;

#[derive(Deserialize, Clone)]
pub(crate) struct LoginData {
    pub token: String,
    /// Seconds until the token expires.
    pub expires: u64,
}

pub enum AuthType {
    Session,
    Basic,
}

impl FromStr for AuthType {
    type Err = TeltonikaError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "session" => Ok(AuthType::Session),
            "basic" => Ok(AuthType::Basic),
            other => Err(TeltonikaError::InvalidConfig(format!(
                "unknown auth type `{other}`, expected `session` or `basic`"
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AuthCredentials {
    pub username: String,
    pub password: String,
}

pub(crate) async fn login(
    http: &reqwest::Client,
    base_url: &str,
    credentials: &AuthCredentials,
) -> Result<LoginData> {
    let response = http
        .post(format!("{base_url}/api/login"))
        .json(credentials)
        .send()
        .await
        .map_err(from_reqwest)?;

    let status = response.status();
    if status == 401 || status == 403 {
        return Err(TeltonikaError::AuthFailed {
            username: credentials.username.clone(),
        });
    }
    let response = response.error_for_status().map_err(from_reqwest)?;

    response
        .json::<Envelope<LoginData>>()
        .await
        .map_err(from_reqwest)?
        .into_data("POST /login")
}

#[derive(Clone)]
pub(crate) enum AuthHeader {
    Session {
        token: String,
        expires_at: Instant,
        credentials: AuthCredentials,
    },
    Basic {
        encoded: String,
        username: String,
    },
}

impl AuthHeader {
    pub(crate) fn session(login_data: LoginData, credentials: AuthCredentials) -> Self {
        Self::Session {
            token: login_data.token,
            expires_at: Instant::now() + Duration::from_secs(login_data.expires),
            credentials,
        }
    }

    pub(crate) fn basic(username: String, password: String) -> Self {
        let encoded = base64_encode(&format!("{username}:{password}"));
        Self::Basic { encoded, username }
    }

    pub(crate) fn value(&self) -> String {
        match self {
            Self::Session { token, .. } => format!("Bearer {token}"),
            Self::Basic { encoded, .. } => format!("Basic {encoded}"),
        }
    }

    pub(crate) fn username(&self) -> &str {
        match self {
            Self::Session { credentials, .. } => &credentials.username,
            Self::Basic { username, .. } => username,
        }
    }

    /// Basic credentials never expire, so this is session-only.
    pub(crate) fn is_stale(&self, margin: Duration) -> bool {
        match self {
            Self::Session { expires_at, .. } => Instant::now() + margin >= *expires_at,
            Self::Basic { .. } => false,
        }
    }

    pub(crate) fn can_refresh(&self) -> bool {
        matches!(self, Self::Session { .. })
    }

    pub(crate) async fn refresh(&mut self, http: &reqwest::Client, base_url: &str) -> Result<()> {
        let Self::Session {
            token,
            expires_at,
            credentials,
        } = self
        else {
            return Ok(());
        };

        let data = login(http, base_url, credentials).await?;
        *token = data.token;
        *expires_at = Instant::now() + Duration::from_secs(data.expires);
        Ok(())
    }
}