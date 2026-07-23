use teltonika_core::device::{Device, Credentials};
use serde::Deserialize;
use reqwest;
use base64;

pub enum AuthType {
    Session(Credentials),
    Basic(Credentials),
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

pub async fn authenticate(auth: AuthType, device: Device) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    match auth {
        AuthType::Session(credentials) => {
            println!("Authenticating with session for user: {}", credentials.username);
            let url = format!("http://{}/api/login", device.ip);
            let response = client.post(url).json(&device.credentials)
                .send()
                .await?;
            if response.status().is_success() {
                let token = response.error_for_status()?.json::<LoginResponse>().await?.token;                Ok(token)
            } else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Session authentication failed.")))
            }
        },
        AuthType::Basic(credentials) => {
            println!("Authenticating with basic auth for user: {}", credentials.username);
            let encoded_credentials = base64_encode(&format!("{}:{}", credentials.username, credentials.password));
            Ok(encoded_credentials)
        },
    }
}

fn base64_encode(input: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, input)
}