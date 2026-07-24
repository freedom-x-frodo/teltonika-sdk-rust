use crate::client::RestClient;

pub mod system;

impl RestClient {
    pub fn system(&self) -> system::SystemApi<'_> {
        system::SystemApi::new(self)
    }
}