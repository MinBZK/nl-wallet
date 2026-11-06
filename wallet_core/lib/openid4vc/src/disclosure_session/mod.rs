use std::collections::HashMap;
use std::hash::Hash;

use attestation_data::organization::Organization;
use attestation_types::credential_format::Format;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::CredentialEcdsaKey;
use crypto::trust_anchor::TrustAnchors;
use crypto::wscd::DisclosureWscd;
use dcql::CredentialQueryIdentifier;
use dcql::normalized::NormalizedCredentialRequests;
use mdoc::holder::disclosure::PartialMdoc;
use nutype::nutype;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use url::Url;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;

pub use self::client::VpDisclosureClient;
pub use self::error::DataDisclosed;
pub use self::error::DisclosureError;
pub use self::error::VpClientError;
pub use self::error::VpSessionError;
pub use self::error::VpVerifierError;
pub use self::message_client::HttpVpMessageClient;
pub use self::message_client::VpMessageClient;
pub use self::message_client::VpMessageClientError;
pub use self::message_client::VpMessageClientErrorType;
pub use self::session::VpDisclosureSession;
pub use self::uri_source::DisclosureUriSource;
use crate::verifier::SessionType;

mod client;
mod error;
mod message_client;
mod session;
mod uri_source;

#[cfg(feature = "mock")]
pub mod mock;

/// Trust anchors for authenticating the verifier and validating its registration certificate.
#[derive(Debug)]
pub struct DisclosureTrustAnchors<'a> {
    pub wrpac: &'a TrustAnchors,
    pub wrprc: &'a TrustAnchors,
}

pub trait DisclosureClient {
    type Session: DisclosureSession;

    async fn start(
        &self,
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        trust_anchors: DisclosureTrustAnchors<'_>,
    ) -> Result<Self::Session, VpSessionError>;
}

/// The attestations to be disclosed in response to a single credential query, along with their key identifiers.
#[derive(Debug, Clone)]
pub enum DisclosableAttestations {
    MsoMdoc(VecNonEmpty<(PartialMdoc, String)>),
    SdJwt(VecNonEmpty<(UnsignedSdJwtPresentation, String)>),
}

impl DisclosableAttestations {
    pub fn format(&self) -> Format {
        match self {
            Self::MsoMdoc(_) => Format::MsoMdoc,
            Self::SdJwt(_) => Format::SdJwt,
        }
    }
}

/// The attestations to be disclosed per credential query. Note that different credential queries may
/// be answered using attestations of different formats.
#[nutype(
    derive(Debug, Clone, AsRef, TryFrom),
    validate(predicate = |attestations| !attestations.is_empty()),
)]
pub struct NonEmptyDisclosableAttestations(HashMap<CredentialQueryIdentifier, DisclosableAttestations>);

impl NonEmptyDisclosableAttestations {
    /// Check that each credential query for which attestations are provided is present in the credential requests,
    /// and that those attestations have the format requested in that credential query.
    fn check_against_credential_requests(
        &self,
        credential_requests: &NormalizedCredentialRequests,
    ) -> Result<(), VpClientError> {
        for (id, attestations) in self.as_ref() {
            let request = credential_requests
                .as_ref()
                .iter()
                .find(|request| request.id() == id)
                .ok_or(VpClientError::UnknownCredentialQuery)?;

            if request.format() != attestations.format() {
                return Err(VpClientError::AttestationFormatMismatch {
                    requested: request.format(),
                    provided: attestations.format(),
                });
            }
        }

        Ok(())
    }
}

pub trait DisclosureSession {
    fn session_type(&self) -> SessionType;
    /// The identifiers of each [`NormalizedCredentialRequest`] returned are guaranteed to be unique.
    fn credential_requests(&self) -> &NormalizedCredentialRequests;
    /// Organization details from the validated registration certificate.
    fn organization(&self) -> &Organization;

    async fn terminate(self) -> Result<Option<Url>, VpSessionError>;
    async fn disclose<K, W>(
        self,
        attestations: NonEmptyDisclosableAttestations,
        wscd: &W,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Option<Url>, (Box<Self>, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K>,
        Self: Sized;
}
