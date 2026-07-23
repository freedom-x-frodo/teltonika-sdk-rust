use std::str::FromStr;
use serde::{Deserialize, Serialize};
use teltonika_core::{Result,TeltonikaError};
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
pub enum AuthState {
    Session { token: String },
    Basic { encoded: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredentials{
    pub username: String, 
    pub password: String
}