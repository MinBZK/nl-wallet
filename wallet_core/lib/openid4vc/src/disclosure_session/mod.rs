use std::hash::Hash;

use rustls_pki_types::TrustAnchor;

use attestation_types::attribute_paths::AttestationAttributePaths;
use crypto::factory::KeyFactory;
use crypto::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use mdoc::holder::Mdoc;
use poa::factory::PoaFactory;
use utils::vec_at_least::VecNonEmpty;

use crate::verifier::SessionType;

pub use self::client::VpDisclosureClient;
pub use self::error::DisclosureError;
pub use self::error::VpClientError;
pub use self::error::VpSessionError;
pub use self::error::VpVerifierError;
pub use self::message_client::HttpVpMessageClient;
pub use self::message_client::VpMessageClient;
pub use self::message_client::VpMessageClientError;
pub use self::message_client::VpMessageClientErrorType;
pub use self::message_client::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
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
    fn requested_attribute_paths(&self) -> &AttestationAttributePaths;
    fn verifier_certificate(&self) -> &VerifierCertificate;

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
    async fn disclose<K, KF>(
        self,
        mdocs: VecNonEmpty<Mdoc>,
        key_factory: &KF,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
        Self: Sized;
}
