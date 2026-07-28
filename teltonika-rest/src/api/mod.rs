use crate::client::RestClient;

pub mod modems;
pub mod system;

impl RestClient {
    pub fn system(&self) -> system::SystemApi<'_> {
        system::SystemApi::new(self)
    }
    pub fn modems(&self) -> modems::ModemsApi<'_> {
        modems::ModemsApi::new(self)
    }
}
