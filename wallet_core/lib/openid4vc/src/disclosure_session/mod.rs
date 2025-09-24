use std::collections::HashMap;
use std::hash::Hash;

use chrono::DateTime;
use chrono::Utc;
use nutype::nutype;
use rustls_pki_types::TrustAnchor;

use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use dcql::CredentialQueryIdentifier;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;
use mdoc::holder::disclosure::PartialMdoc;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use utils::generator::Generator;
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

#[derive(Debug, Clone)]
pub enum DisclosableAttestations {
    MsoMdoc(HashMap<CredentialQueryIdentifier, VecNonEmpty<PartialMdoc>>),
    SdJwt(HashMap<CredentialQueryIdentifier, VecNonEmpty<(UnsignedSdJwtPresentation, String)>>),
}

#[nutype(
    derive(Debug, Clone, AsRef, TryFrom),
    validate(predicate = |response| match response {
        DisclosableAttestations::MsoMdoc(map) => !map.is_empty(),
        DisclosableAttestations::SdJwt(map) => !map.is_empty(),
    }),
)]
pub struct NonEmptyDisclosableAttestations(DisclosableAttestations);

pub trait DisclosureSession {
    fn session_type(&self) -> SessionType;
    /// The identifiers of each [`NormalizedCredentialRequest`] returned are guaranteed to be unique.
    fn credential_requests(&self) -> &NormalizedCredentialRequests;
    fn verifier_certificate(&self) -> &VerifierCertificate;

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
    async fn disclose<K, W>(
        self,
        attestations: NonEmptyDisclosableAttestations,
        wscd: &W,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K, Poa = Poa>,
        Self: Sized;
}
