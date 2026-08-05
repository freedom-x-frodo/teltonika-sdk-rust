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

/// Every field in both schemas is declared `required` **and** `string`, and the
/// device blanks values it has no reading for rather than omitting them. So
/// `""` is the normal no-data case on the live path, not a defensive branch:
/// without it every no-fix response is a hard decode error.
///
/// `null`, `""` and `"N/A"` map to `None`. A JSON number is accepted for
/// firmware drift but never occurs on 7.23.7. Anything else that fails to
/// parse stays a decode error, so schema changes surface instead of silently
/// becoming `None`.
///
/// Roundtrips through `serde_json::Value` and allocates per annotated field.
/// Fine at the poll rate; never on a hot path.
fn lenient_num<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned + FromStr,
    <T as FromStr>::Err: Display,
{
    use serde::de::Error as _;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) if s.trim().is_empty() || s == "N/A" => Ok(None),
        serde_json::Value::String(s) => s.trim().parse().map(Some).map_err(D::Error::custom),
        other => serde_json::from_value(other)
            .map(Some)
            .map_err(D::Error::custom),
    }
}

fn lenient_str<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() || s == "N/A" => Ok(None),
        other => Ok(other),
    }
}

/// `"1"` and `"0"` are both confirmed on sibling fields of the same object.
/// Any other token yields `None`: for `enabled`, "unparseable" and "receiver
/// off" must not collapse to the same value.
fn flag(raw: Option<&String>) -> Option<bool> {
    match raw?.as_str() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct GpsGlobal {
    #[serde(default, deserialize_with = "lenient_str")]
    pub enabled: Option<String>,
    #[serde(default, deserialize_with = "lenient_str")]
    pub galileo_sup: Option<String>,
    #[serde(default, deserialize_with = "lenient_str")]
    pub glonass_sup: Option<String>,
    #[serde(default, deserialize_with = "lenient_str")]
    pub beidou_sup: Option<String>,
    /// Dynamic Power Optimisation.
    #[serde(default, deserialize_with = "lenient_str")]
    pub dpo_enabled: Option<String>,
    #[serde(default, deserialize_with = "lenient_str")]
    pub mode: Option<String>,

    /// Units unstated by the spec — seconds is likely, milliseconds is
    /// possible. Not named `_s` and not converted until a live value confirms
    /// it; both fields came back blank on a receiver that was running.
    #[serde(default, deserialize_with = "lenient_num")]
    pub interval: Option<u32>,
    #[serde(default, deserialize_with = "lenient_num")]
    pub timeout: Option<u32>,
}

impl GpsGlobal {
    pub fn is_enabled(&self) -> Option<bool> {
        flag(self.enabled.as_ref())
    }

    pub fn is_galileo_supported(&self) -> Option<bool> {
        flag(self.galileo_sup.as_ref())
    }

    pub fn is_glonass_supported(&self) -> Option<bool> {
        flag(self.glonass_sup.as_ref())
    }

    pub fn is_beidou_supported(&self) -> Option<bool> {
        flag(self.beidou_sup.as_ref())
    }

    pub fn is_dpo_enabled(&self) -> Option<bool> {
        flag(self.dpo_enabled.as_ref())
    }
}

/// A GPS fix as the device reports it. **Raw device units and conventions — no
/// REP-103 conversion happens here.** Converting is the caller's job at the ROS
/// boundary:
///
/// - `angle_deg` is a compass heading in degrees, clockwise from north.
///   REP-103 yaw is radians, counter-clockwise from +x/east:
///   `yaw_rad = wrap_pi(FRAC_PI_2 - angle_deg.to_radians())`. Not a unit scale.
/// - `speed` units are unstated; RutOS UIs commonly show km/h. Not named
///   `speed_mps` and not scaled — mislabelled it is a silent 3.6× error.
///
/// Both timestamps are wall clock from the router, subject to jump and NTP
/// step. Never derive a `dt` or a timeout from them; hold an `Instant`
/// alongside the sample for staleness.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct GpsPosition {
    /// **Not a validity signal.** Reads `"0"` with no fix and no satellites,
    /// which a threshold check accepts as a perfect fix. Gate on
    /// [`GpsPosition::position_deg`]; never on this field. Metres assumed,
    /// unconfirmed, and possibly HDOP-derived rather than a distance.
    #[serde(default, deserialize_with = "lenient_num")]
    pub accuracy: Option<f64>,

    /// Raw fix state. `"0"` is confirmed no-fix; the rest of the enum is
    /// undocumented and unobserved, which is why there is no `has_fix()` —
    /// a `!= "0"` test would report every unknown token as a valid fix.
    #[serde(default, deserialize_with = "lenient_str")]
    pub fix_status: Option<String>,

    /// Metres. Datum (MSL vs WGS-84 ellipsoid) unstated. Zero-filled with no
    /// fix.
    #[serde(default, deserialize_with = "lenient_num")]
    pub altitude: Option<f64>,

    /// Units unresolved; zero-filled with no fix.
    #[serde(default, deserialize_with = "lenient_num")]
    pub speed: Option<f64>,

    /// Degrees, WGS-84.
    #[serde(rename = "latitude", default, deserialize_with = "lenient_num")]
    pub latitude_deg: Option<f64>,
    #[serde(rename = "longitude", default, deserialize_with = "lenient_num")]
    pub longitude_deg: Option<f64>,

    /// Compass heading, degrees clockwise from north.
    #[serde(rename = "angle", default, deserialize_with = "lenient_num")]
    pub angle_deg: Option<f64>,

    /// Zero-filled with no fix, so `Some(0)` does not imply a reading.
    #[serde(default, deserialize_with = "lenient_num")]
    pub satellites: Option<u8>,

    /// Device-local wall clock. Format unspecified by the spec and observed
    /// only as the `"0"` zero-fill, so kept raw — the workspace has no
    /// date/time dependency.
    #[serde(default, deserialize_with = "lenient_str")]
    pub timestamp: Option<String>,
    /// UTC wall clock. Same caveat.
    #[serde(default, deserialize_with = "lenient_str")]
    pub utc_timestamp: Option<String>,
}

impl GpsPosition {
    /// `(latitude_deg, longitude_deg)` only when the device reported both and
    /// did not report the confirmed no-fix state. The only validity gate on
    /// this type.
    ///
    /// Fail-closed on the coordinates, which the device blanks without a fix.
    /// Still fail-open on an unrecognised `fix_status`: tighten to an allowlist
    /// once the non-zero tokens are observed on a device with a fix.
    pub fn position_deg(&self) -> Option<(f64, f64)> {
        if self.fix_status.as_deref() == Some("0") {
            return None;
        }
        let latitude_deg = self.latitude_deg?;
        let longitude_deg = self.longitude_deg?;
        if !latitude_deg.is_finite()
            || !longitude_deg.is_finite()
            || latitude_deg.abs() > 90.0
            || longitude_deg.abs() > 180.0
        {
            return None;
        }
        Some((latitude_deg, longitude_deg))
    }
}