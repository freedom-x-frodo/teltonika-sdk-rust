use std::str::FromStr;
use serde::{Deserialize, Serialize};

pub enum AuthType {
    Session,
    Basic,
}

impl FromStr for AuthType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(AuthType::Session),
            "basic" => Ok(AuthType::Basic),
            _  => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredentials{
    pub username: String, 
    pub password: String
}