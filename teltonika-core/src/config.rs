use std::time::Duration;

pub struct ConnConfig {
    pub username: String,
    pub password: String,
    pub endpoint: String,
    pub accept_invalid_certs: bool,
    pub refresh_margin: Duration,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

impl ConnConfig {
    pub fn new(username: String, password: String, endpoint: String) -> Self {
        Self {
            username,
            password,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            accept_invalid_certs: true,
            refresh_margin: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(15),
        }
    }

    pub fn with_refresh_margin(mut self, refresh_margin: Duration) -> Self {
        self.refresh_margin = refresh_margin;
        self
    }

    pub fn with_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Applies per HTTP request, not per SDK call: a `get()` that triggers a
    /// token refresh and a 401 retry issues several requests, each bounded
    /// separately.
    pub fn with_request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }
}