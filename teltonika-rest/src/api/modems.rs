use serde::Deserialize;
use serde::de::DeserializeOwned;

use teltonika_core::Result;

use crate::client::RestClient;

pub struct ModemsApi<'a> {
    client: &'a RestClient,
}

impl<'a> ModemsApi<'a> {
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// `GET /modems/status`. Array; one modem per element. Offline modems come
    /// back as a sparse stub, so every field below is optional.
    pub async fn status(&self) -> Result<Vec<ModemStatus>> {
        self.client.get("/modems/status").await
    }
}

/// RutOS fills unavailable numerics with the string `"N/A"` rather than omitting
/// them or sending null. Confirmed on RG501Q-EU / RutOS 7.23.7 under 5G-NSA: the
/// NR component carrier in `cell_info` reports `nr-arfcn` and leaves
/// `earfcn: "N/A"`, which failed the whole response and latched the node in
/// STATE_BAD_RESPONSE for as long as the modem stayed on 5G-NSA. `lac`, `tac`,
/// `arfcn` and `uarfcn` use the same convention in the same object.
///
/// Only the exact `"N/A"` token maps to None. Any other unexpected type is still
/// a decode error, so genuine schema drift keeps surfacing rather than being
/// silently swallowed.
///
/// Roundtrips through `serde_json::Value`, so each annotated field allocates
/// during decode. Fine at the poll rate; do not copy this onto a hot path.
fn na_num<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned,
{
    use serde::de::Error as _;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) if s == "N/A" => Ok(None),
        other => serde_json::from_value(other)
            .map(Some)
            .map_err(D::Error::custom),
    }
}

// `default` is load-bearing next to every `deserialize_with`: without it serde
// overrides Option's implicit default and a *missing* field becomes an error.
// The second `ca_signal` element on 5G-NSA omits every radio field.

#[derive(Clone, Deserialize)]
#[allow(dead_code)]
pub struct ModemStatus {
    pub state: Option<String>,
    pub id: Option<String>,
    pub ntype: Option<String>, // network type, e.g. "5G-NSA"
    provider: Option<String>,
    pub conntype: Option<String>,
    pub band: Option<String>,

    cellid: Option<String>,

    // `imsi` is deliberately NOT declared. Nothing in this workspace consumes
    // it, and serde drops undeclared fields -- which is what already keeps
    // `imei`, `iccid` and `serial` out of this struct and out of any log. Do not
    // add it back without a consumer; if SMS needs it, wrap it so `Debug`
    // cannot print it.
    #[serde(default, deserialize_with = "na_num")]
    pub signal: Option<i16>, // probably in dB
    #[serde(default, deserialize_with = "na_num")]
    pub signal_quality: Option<i16>, // separate from `signal`; unit unconfirmed
    #[serde(default, deserialize_with = "na_num")]
    pub rsrp: Option<i16>, // reference signal received power, dBm
    #[serde(default, deserialize_with = "na_num")]
    pub rsrq: Option<i16>, // reference signal received quality, dB
    #[serde(default, deserialize_with = "na_num")]
    pub rssi: Option<i16>, // received signal strength, dBm

    /// Celsius, unscaled -- live RG501Q-EU reports 45, not 450.
    #[serde(default, deserialize_with = "na_num")]
    pub temperature: Option<f32>,

    pub data_conn_state: Option<String>,
    /// Spec domain is {0, 1, 2} with no labels. A live device reports 1 for
    /// "Connected"; 0 and 2 are unverified. Prefer `is_connected()`.
    #[serde(default, deserialize_with = "na_num")]
    pub data_conn_state_id: Option<i16>,
    busy_state: Option<String>,
    #[serde(default, deserialize_with = "na_num")]
    busy_state_id: Option<i16>,
    #[serde(default, deserialize_with = "na_num")]
    pub mobile_stage: Option<i16>,

    // Not sentinel-wrapped: no evidence the device reports "N/A" for a byte
    // counter or the serial rate. Annotating a field with no evidence behind it
    // hides which fields were actually confirmed.
    baudrate: Option<i32>,
    rxbytes: Option<i64>,
    txbytes: Option<i64>,

    pub ca_signal: Option<Vec<SignalInfo>>, // per carrier-aggregation component
    cell_info: Option<Vec<CellInfo>>,
}

/// Manual so cell identity cannot reach a log by accident.
/// `examples/modems.rs` prints `{modem:#?}`; on a fleet robot that is a durable
/// artifact, and `cellid` + `mnc` + band is a coarse position fix.
impl std::fmt::Debug for ModemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModemStatus")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("ntype", &self.ntype)
            .field("provider", &self.provider)
            .field("conntype", &self.conntype)
            .field("band", &self.band)
            .field("cellid", &self.cellid.as_ref().map(|_| "<redacted>"))
            .field("signal", &self.signal)
            .field("signal_quality", &self.signal_quality)
            .field("rsrp", &self.rsrp)
            .field("rsrq", &self.rsrq)
            .field("rssi", &self.rssi)
            .field("temperature", &self.temperature)
            .field("data_conn_state", &self.data_conn_state)
            .field("data_conn_state_id", &self.data_conn_state_id)
            .field("busy_state", &self.busy_state)
            .field("busy_state_id", &self.busy_state_id)
            .field("mobile_stage", &self.mobile_stage)
            .field("baudrate", &self.baudrate)
            .field("rxbytes", &self.rxbytes)
            .field("txbytes", &self.txbytes)
            .field("ca_signal", &self.ca_signal)
            .field("cell_info", &self.cell_info)
            .finish()
    }
}

/// One component carrier under carrier aggregation. `primary` marks the anchor.
/// On 5G-NSA the non-primary NR entry carries only band, bandwidth and
/// frequency -- the radio fields are absent, not `"N/A"`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SignalInfo {
    pub primary: Option<bool>,
    pub band: Option<String>,
    pub bandwidth: Option<String>,
    /// EARFCN on an LTE carrier, NR-ARFCN on an NR carrier -- two numbering
    /// spaces in one field. Disambiguate with `band` before comparing values.
    #[serde(default, deserialize_with = "na_num")]
    pub frequency: Option<i32>,
    #[serde(default, deserialize_with = "na_num")]
    pub pcid: Option<i32>, // physical cell id
    #[serde(default, deserialize_with = "na_num")]
    pub rssi: Option<i16>,
    #[serde(default, deserialize_with = "na_num")]
    pub rsrp: Option<i16>,
    #[serde(default, deserialize_with = "na_num")]
    pub rsrq: Option<i16>,
    #[serde(default, deserialize_with = "na_num")]
    pub sinr: Option<i16>, // signal-to-interference+noise, dB
}

/// Serving-cell fields. Most of this schema comes back `"N/A"` depending on
/// which radio technology the carrier is using.
#[derive(Clone, Deserialize)]
#[allow(dead_code)]
struct CellInfo {
    cellid: Option<String>,
    mnc: Option<String>, // mobile network code (carrier)
    band: Option<String>,
    bandwidth: Option<String>,
    /// `"N/A"` on an NR component carrier, which reports `nr-arfcn` instead.
    /// This is the field that broke decoding under 5G-NSA.
    #[serde(default, deserialize_with = "na_num")]
    earfcn: Option<i32>, // channel number; NR range needs i32
    #[serde(default, deserialize_with = "na_num")]
    pcid: Option<i32>,
    #[serde(default, deserialize_with = "na_num")]
    sinr: Option<i16>,
    #[serde(default, deserialize_with = "na_num")]
    ue_state: Option<i16>, // UE = the modem; RRC connection state
}

/// `cellid` and `mnc` together are location-revealing; kept out of logs.
impl std::fmt::Debug for CellInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellInfo")
            .field("cellid", &self.cellid.as_ref().map(|_| "<redacted>"))
            .field("mnc", &self.mnc.as_ref().map(|_| "<redacted>"))
            .field("band", &self.band)
            .field("bandwidth", &self.bandwidth)
            .field("earfcn", &self.earfcn)
            .field("pcid", &self.pcid)
            .field("sinr", &self.sinr)
            .field("ue_state", &self.ue_state)
            .finish()
    }
}

impl ModemStatus {
    /// True only on the device's explicit "Connected" state. "Disconnected" and
    /// "Unknown" are both not-connected, and so is an absent field.
    ///
    /// Keyed on the string, not `data_conn_state_id`: the spec's id enum is
    /// {0, 1, 2} with no labels, and the parallel string/id enums in this spec
    /// are not positionally aligned (`pinstate` has 8 strings to 9 ids), so the
    /// mapping cannot be recovered by position. The string enum is closed and
    /// exhaustive in the spec, and a live device agrees with it.
    ///
    /// This is data-connection state, not reachability --
    /// `/internet_connection/status` is the separate question.
    pub fn is_connected(&self) -> bool {
        self.data_conn_state.as_deref() == Some("Connected")
    }
}