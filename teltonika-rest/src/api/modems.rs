use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ModemStatus {
    state: Option<String>,
    id: Option<String>,
    ntype: Option<String>,             // network type, e.g. "5G-NSA"
    provider: Option<String>,
    conntype: Option<String>,
    band: Option<String>,
    cellid: Option<String>,
    imsi: Option<String>,              // subscriber id, tied to SIM - sensitive

    // rolled-up signal; unit unconfirmed (bars vs dBm) - check live dump
    signal: Option<i16>,
    signal_quality: Option<i16>,       // separate from `signal`; likely 0-100%
    rsrp: Option<i16>,                 // reference signal received power, dBm
    rsrq: Option<i16>,                 // reference signal received quality, dB
    rssi: Option<i16>,                 // received signal strength, dBm

    temperature: Option<f32>,          // Celsius; may arrive x10 - verify

    data_conn_state: Option<String>,
    data_conn_state_id: Option<i16>,   // machine-readable pair of the above
    busy_state: Option<String>,
    busy_state_id: Option<i16>,
    mobile_stage: Option<i16>,

    baudrate: Option<i32>,
    rxbytes: Option<i64>,
    txbytes: Option<i64>,

    ca_signal: Option<Vec<SignalInfo>>, // per carrier-aggregation component
    cell_info: Option<Vec<CellInfo>>,
}

// One component carrier under carrier aggregation. `primary` marks the anchor.
#[derive(Debug, Clone, Deserialize)]
struct SignalInfo {
    primary: Option<bool>,
    band: Option<String>,
    bandwidth: Option<String>,
    frequency: Option<i32>,
    pcid: Option<i32>,                 // physical cell id
    rssi: Option<i16>,
    rsrp: Option<i16>,
    rsrq: Option<i16>,
    sinr: Option<i16>,                 // signal-to-interference+noise, dB
}

// Serving-cell fields; the rest of the schema comes back N/A on LTE.
#[derive(Debug, Clone, Deserialize)]
struct CellInfo {
    cellid: Option<String>,
    mnc: Option<String>,               // mobile network code (carrier)
    band: Option<String>,
    bandwidth: Option<String>,
    earfcn: Option<i32>,               // channel number; NR range needs i32
    pcid: Option<i32>,
    sinr: Option<i16>,
    ue_state: Option<i16>,             // UE = the modem; RRC connection state
}

impl ModemStatus {
    /// True once the modem reaches a usable data connection.
    pub fn is_connected(&self) -> bool {
        self.data_conn_state_id == Some(3) // guess - confirm id on live dump
    }
}