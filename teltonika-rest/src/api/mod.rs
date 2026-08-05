use crate::client::RestClient;

pub mod modems;
pub mod system;
pub mod gps;

impl RestClient {
    pub fn system(&self) -> system::SystemApi<'_> {
        system::SystemApi::new(self)
    }
    pub fn modems(&self) -> modems::ModemsApi<'_> {
        modems::ModemsApi::new(self)
    }
    pub fn gps(&self) -> gps::GpsApi<'_> {
        gps::GpsApi::new(self)
    }
}
