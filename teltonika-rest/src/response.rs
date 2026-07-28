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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_success() {
        let env = Envelope {
            success: true,
            data: Some(7u32),
        };
        assert_eq!(env.into_data("GET /x").unwrap(), 7);
    }

    #[test]
    fn rejects_failure_flag() {
        let env = Envelope::<u32> {
            success: false,
            data: None,
        };
        let err = env.into_data("GET /x").unwrap_err().to_string();
        assert!(err.contains("GET /x"));
    }

    #[test]
    fn rejects_missing_data() {
        let env = Envelope::<u32> {
            success: true,
            data: None,
        };
        assert!(matches!(
            env.into_data("GET /x"),
            Err(TeltonikaError::InvalidResponse(_))
        ));
    }
}
