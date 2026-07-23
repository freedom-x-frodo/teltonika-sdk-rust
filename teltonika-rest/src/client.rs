use serde::Deserialize;
use reqwest;
use std::sync::Arc;
use std::str::FromStr;

use teltonika_core::config::ConnConfig;
use crate::auth::{AuthType, AuthCredentials};
use crate::utils::base64_encode;

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Clone)]
pub struct RestClient {
    inner: Arc<ClientInner>,
}
pub struct ClientInner{
    auth_type: AuthType,
    token: String
}

impl RestClient {
    pub fn new(auth_type: String) -> Result<RestClient, ()> {
        let auth_type = AuthType::from_str(&auth_type)?;
        let token = String::new();
        let inner = Arc::new(ClientInner{auth_type, token});
        Ok(RestClient{ inner})
    }
    pub async fn auth(&self, config: ConnConfig) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let credentials = AuthCredentials{
            username: config.username,
            password: config.password
        };
        let url = format!("http://{}/api/login", config.ip);
        match self.inner.auth_type {
            AuthType::Session => {
                println!("Authenticating with session for user: {}", credentials.username);

                let response = client.post(url).json(&credentials)
                    .send()
                    .await?;
                if response.status().is_success() {
                    let token = response.error_for_status()?.json::<LoginResponse>().await?.token;                
                    self.inner.token = token;
                    Ok(())
                } else {
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Session authentication failed.")))
                }
            },
            AuthType::Basic => {
                println!("Authenticating with basic auth for user: {}", credentials.username);
                let encoded_credentials = base64_encode(&format!("{}:{}", credentials.username, credentials.password));
                self.inner.token = encoded_credentials;
                Ok(())
            },
        }
    }

}

