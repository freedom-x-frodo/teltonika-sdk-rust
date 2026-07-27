use serde::Deserialize;

use teltonika_core::{Result, TeltonikaError};

/// Standard RutOS response wrapper. Login uses it too — the vendored spec
/// carries `success` there.
#[derive(Deserialize)]
pub(crate) struct Envelope<T> {
    success: bool,
    data: Option<T>,
}

impl<T> Envelope<T> {
    /// `context` names the request for error messages, e.g. `"GET /modems/status"`.
    pub(crate) fn into_data(self, context: &str) -> Result<T> {
        if !self.success {
            return Err(TeltonikaError::InvalidResponse(format!(
                "device reported failure for {context}"
            )));
        }
        self.data.ok_or_else(|| {
            TeltonikaError::InvalidResponse(format!("missing `data` in response to {context}"))
        })
    }
}