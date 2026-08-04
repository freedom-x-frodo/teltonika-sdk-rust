use teltonika_core::TeltonikaError;

pub(crate) fn from_reqwest(err: reqwest::Error) -> TeltonikaError {
    if let Some(status) = err.status() {
        TeltonikaError::Http { status: status.as_u16() }
    } else if err.is_timeout() || err.is_connect() {
        TeltonikaError::Network(with_causes(&err))
    } else if err.is_decode() {
        TeltonikaError::InvalidResponse(with_causes(&err))
    } else {
        TeltonikaError::Network(with_causes(&err))
    }
}

fn with_causes(err: &dyn std::error::Error) -> String {
    let mut s = err.to_string();
    let mut src = err.source();
    while let Some(e) = src {
        s.push_str(": ");
        s.push_str(&e.to_string());
        src = e.source();
    }
    s.truncate(512);
    s
}