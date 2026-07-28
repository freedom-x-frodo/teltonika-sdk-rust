use std::time::Duration;

pub struct ConnConfig {
    pub username: String,
    pub password: String,
    pub endpoint: String,
    pub accept_invalid_certs: bool,
    pub refresh_margin: Duration,
}

impl ConnConfig {
    pub fn new(username: String, password: String, endpoint: String) -> Self {
        Self {
            username,
            password,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            accept_invalid_certs: true,
            refresh_margin: Duration::from_secs(30),
        }
    }

    pub fn with_refresh_margin(mut self, refresh_margin: Duration) -> Self {
        self.refresh_margin = refresh_margin;
        self
    }
}