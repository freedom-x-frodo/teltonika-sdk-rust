use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TeltonikaError {
    /// Device rejected the credentials (HTTP 401/403 on login).
    #[error("authentication failed for user `{username}`")]
    AuthFailed { username: String },

    /// Transport-level failure: DNS, connect refused, timeout, TLS.
    #[error("network error: {0}")]
    Network(String),

    /// Device answered, but the body wasn't what we expected.
    #[error("invalid response from device: {0}")]
    InvalidResponse(String),

    /// Device returned an API-level error status.
    #[error("device returned HTTP {status}")]
    Http { status: u16 },

    /// Configuration is malformed (bad host, unknown auth type, ...).
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, TeltonikaError>;