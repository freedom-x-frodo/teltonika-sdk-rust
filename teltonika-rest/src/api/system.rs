use crate::client::RestClient;

pub struct SystemApi<'a> {
    client: &'a RestClient,
}

impl<'a> SystemApi<'a> {
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }
}