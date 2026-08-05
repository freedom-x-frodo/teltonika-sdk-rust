// teltonika-rest/src/api/gps.rs
use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use teltonika_core::Result;

use crate::client::RestClient;

/// `/gps/...` endpoints. Obtained via [`RestClient::gps`].
pub struct GpsApi<'a> {
    client: &'a RestClient,
}

impl<'a> GpsApi<'a> {
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// `GET /gps/global` — receiver configuration. Static between config
    /// writes; poll once at startup, not on the telemetry interval.
    pub async fn global(&self) -> Result<GpsGlobal> {
        self.client.get("/gps/global").await
    }

    /// `GET /gps/position/status` — current fix.
    pub async fn position_status(&self) -> Result<GpsPosition> {
        self.client.get("/gps/position/status").await
    }
}

fn lenient_num<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned + FromStr,
    <T as FromStr>::Err: Display,
{
    use serde::de::Error as _;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) if s.is_empty() || s == "N/A" => Ok(None),
        serde_json::Value::String(s) => s.trim().parse().map(Some).map_err(D::Error::custom),
        other => serde_json::from_value(other)
            .map(Some)
            .map_err(D::Error::custom),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct GpsGlobal {
    pub enabled: Option<String>,
    pub galileo_sup: Option<String>,
    pub glonass_sup: Option<String>,
    pub beidou_sup: Option<String>,
    pub dpo_enabled: Option<String>,
    pub mode: Option<String>,

    #[serde(default, deserialize_with = "lenient_num")]
    pub interval: Option<u32>,
    #[serde(default, deserialize_with = "lenient_num")]
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct GpsPosition {
    /// Reported accuracy. Metres assumed, unconfirmed; may be an HDOP-derived
    /// figure rather than a distance.
    #[serde(default, deserialize_with = "lenient_num")]
    pub accuracy: Option<f64>,

    
    pub fix_status: Option<String>,

    #[serde(default, deserialize_with = "lenient_num")]
    pub altitude: Option<f64>,

    /// See type-level note — units unresolved.
    #[serde(default, deserialize_with = "lenient_num")]
    pub speed: Option<f64>,

    /// Degrees, WGS-84.
    #[serde(default, deserialize_with = "lenient_num")]
    pub latitude_deg: Option<f64>,
    #[serde(default, deserialize_with = "lenient_num")]
    pub longitude_deg: Option<f64>,

    /// Compass heading, degrees clockwise from north. See type-level note.
    #[serde(default, deserialize_with = "lenient_num")]
    pub angle_deg: Option<f64>,

    #[serde(default, deserialize_with = "lenient_num")]
    pub satellites: Option<u8>,

    /// Device-local wall clock, format unspecified by the spec — kept as the
    /// raw string, since the workspace has no date/time dependency.
    pub timestamp: Option<String>,
    /// UTC wall clock. Same caveat.
    pub utc_timestamp: Option<String>,
}