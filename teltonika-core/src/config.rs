use std::time::Duration;

pub struct ConnConfig {
    pub username: String,
    pub password: String,
    pub ip: String,
    pub accept_invalid_certs: bool,
    pub refresh_margin: Duration, //expiration time renewal for session login
}

impl ConnConfig {
    pub fn new(username: String, password: String, ip: String) -> Self {
        Self {
            username,
            password,
            ip,
            accept_invalid_certs: true,
            refresh_margin: Duration::from_secs(30),
        }
    }

    pub fn with_refresh_margin(mut self, refresh_margin: Duration) -> Self {
        self.refresh_margin = refresh_margin;
        self
    }
}