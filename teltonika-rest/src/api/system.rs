use serde::Deserialize;

use teltonika_core::Result;

use crate::client::RestClient;

/// `/system/...` endpoints. Obtained via [`RestClient::system`].
pub struct SystemApi<'a> {
    client: &'a RestClient,
}

impl<'a> SystemApi<'a> {
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// `GET /system/device/usage/status` — dynamic device health.
    pub async fn usage_status(&self) -> Result<UsageStatus> {
        self.client.get("/system/device/usage/status").await
    }
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct UsageStatus {
    pub memory: Option<Memory>,
    pub load: Option<Load>,

    /// Human-formatted, e.g. `"07h 56m 02s"`. Prefer `uptime_seconds`.
    pub uptime: Option<String>,
    pub uptime_seconds: Option<u64>,

    pub loadavg: Option<f64>,

    /// Device local time as a Unix timestamp. Raw — no conversion, the
    /// workspace has no date/time dependency.
    pub localtime: Option<i64>,
}

/// All values in MB except the two `_percentage` fields.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Memory {
    pub ram_total: Option<f64>,
    pub ram_used: Option<f64>,
    pub ram_free: Option<f64>,
    pub ram_shared: Option<f64>,
    pub ram_buffered: Option<f64>,
    pub ram_percentage: Option<f64>,

    pub flash_total: Option<f64>,
    pub flash_used: Option<f64>,
    pub flash_free: Option<f64>,
    pub flash_percentage: Option<f64>,
}

/// CPU load over 1/5/15 minutes.
///
/// **Units unresolved.** The spec says percent, but its two examples differ by
/// ~4 orders of magnitude (`min1: 4800` vs `min1: 0.0766`) — the latter is
/// ordinary Unix loadavg, the former looks like raw kernel fixed-point.
/// Deliberately not scaled or renamed until a live dump settles it.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Load {
    pub min1: Option<f64>,
    pub min5: Option<f64>,
    pub min15: Option<f64>,
}