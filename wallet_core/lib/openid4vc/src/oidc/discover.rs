use crate::issuer_identifier::IssuerIdentifier;
use derive_more::AsRef;

use super::OidcReqwestClient;

pub trait Discover<M, E> {
    async fn discover(&self, identifier: &IssuerIdentifier) -> Result<M, E>;
}

/// Implementation that performs HTTP discovery.
#[derive(Debug, AsRef)]
pub struct HttpDiscover(OidcReqwestClient);

impl HttpDiscover {
    pub fn new(client: OidcReqwestClient) -> Self {
        Self(client)
    }
}
