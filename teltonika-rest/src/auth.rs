use std::str::FromStr;
use serde::{Deserialize, Serialize};
use teltonika_core::{Result,TeltonikaError};

#[derive(Deserialize)]
pub(crate) struct LoginResponse {
    pub data: LoginData,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LoginData {
    pub token: String,
    pub expires: Option<u64>, // TODO: use for re-auth scheduling
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
            other => Err(TeltonikaError::InvalidConfig(
                format!("unknown auth type `{other}`, expected `session` or `basic`"),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AuthState {
    Session { login_data: LoginData },
    Basic { encoded: String },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AuthCredentials{
    pub username: String, 
    pub password: String
}