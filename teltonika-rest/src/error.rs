use teltonika_core::TeltonikaError;

pub(crate) fn from_reqwest(err: reqwest::Error) -> TeltonikaError {
    if let Some(status) = err.status() {
        TeltonikaError::Http {
            status: status.as_u16(),
        }
    } else if err.is_timeout() || err.is_connect() {
        TeltonikaError::Network(err.to_string())
    } else if err.is_decode() {
        TeltonikaError::InvalidResponse(err.to_string())
    } else {
        TeltonikaError::Network(err.to_string())
    }
}
