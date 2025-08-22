use std::hash::Hash;

use rustls_pki_types::TrustAnchor;

use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;
use mdoc::holder::disclosure::PartialMdoc;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;

use crate::verifier::SessionType;

pub use self::client::VpDisclosureClient;
pub use self::error::DisclosureError;
pub use self::error::VpClientError;
pub use self::error::VpSessionError;
pub use self::error::VpVerifierError;
pub use self::message_client::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
pub use self::message_client::HttpVpMessageClient;
pub use self::message_client::VpMessageClient;
pub use self::message_client::VpMessageClientError;
pub use self::message_client::VpMessageClientErrorType;
pub use self::session::VpDisclosureSession;
pub use self::uri_source::DisclosureUriSource;
pub use self::verifier_certificate::VerifierCertificate;

mod client;
mod error;
mod message_client;
mod session;
mod uri_source;
mod verifier_certificate;

#[cfg(feature = "mock")]
pub mod mock;

pub trait DisclosureClient {
    type Session: DisclosureSession;

    async fn start(
        &self,
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Session, VpSessionError>;
}

pub trait DisclosureSession {
    fn session_type(&self) -> SessionType;
    /// The identifiers of each [`NormalizedCredentialRequest`] returned are guaranteed to be unique.
    fn credential_requests(&self) -> &NormalizedCredentialRequests;
    fn verifier_certificate(&self) -> &VerifierCertificate;

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
    async fn disclose<K, W>(
        self,
        partial_mdocs: VecNonEmpty<PartialMdoc>,
        wscd: &W,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K, Poa = Poa>,
        Self: Sized;
}
