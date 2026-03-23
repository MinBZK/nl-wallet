use crate::issuer_identifier::IssuerIdentifier;
use derive_more::AsRef;

use super::HttpJsonClient;

pub trait Discover<M, E> {
    async fn discover(&self, identifier: &IssuerIdentifier) -> Result<M, E>;
}

/// Implementation that performs HTTP discovery.
#[derive(Debug, AsRef)]
pub struct HttpDiscover(HttpJsonClient);

impl HttpDiscover {
    pub fn new(client: HttpJsonClient) -> Self {
        Self(client)
    }
}
