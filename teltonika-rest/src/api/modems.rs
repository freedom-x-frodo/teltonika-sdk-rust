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

    /// `GET /modems/status` - very huge payload on modems status.
    /// but we have only one modem for now so there is only one modem info in data provided.
    pub async fn status(&self) -> Result<Vec<ModemStatus>> {
        self.client.get("/modems/status").await
    }
}   

#[derive(Debug, Clone, Deserialize)]
pub struct ModemStatus {
    state: String,
    id: String,
    ntype: String, // network type
    provider: String,
    ca_signal: Vec<SignalInfo>, //i am not sure what is CA there means
    signal: i16,
    data_conn_state_id: i16,
    data_conn_state: String,
    rsrp: i8,
    cellid: String,
    cell_info: Vec<CellInfo>,
    temperature: f32, //not sure if temp is in C
    conntype: String,
    rssi: i8,
    imsi: String,
    signal_quality: i16,
    rsrq: i8,
    band: String,
    mobile_stage: i16,
    busy_state_id: i8,
    busy_state: String,
    baudrate: i32,
    rxbytes: i64,
    txbytes: i64
}   

#[derive(Debug, Clone, Deserialize)]
struct SignalInfo{
    primary: bool,
    rssi: i8, //worried if it is enough if max rssi val is 255
    bandwidth: String,
    sinr: i8, //same worry
    rsrq: i8,
    rsrp: i8,
    pcid: i32,
    band: String,
    frequency: i32
}

// most of the fields from schema are N/A from response 
// so I also did not impelment it here neither
#[derive(Debug, Clone, Deserialize)]
struct CellInfo{
    cellid: String,
    mnc: String,
    earfcn: i16,
    bandwidth: String,
    sinr: i8,
    pcid: i16,
    ue_state: i8
}