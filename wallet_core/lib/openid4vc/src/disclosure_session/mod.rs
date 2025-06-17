use std::hash::Hash;

use http_utils::urls::BaseUrl;
use rustls_pki_types::TrustAnchor;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::identifiers::AttributeIdentifier;
use crypto::factory::KeyFactory;
use crypto::x509::BorrowingCertificate;
use crypto::CredentialEcdsaKey;
use mdoc::holder::MdocDataSource;
use mdoc::holder::ProposedAttributes;
use poa::factory::PoaFactory;

use crate::verifier::SessionType;

pub use self::client::HttpVpMessageClient;
pub use self::client::VpMessageClient;
pub use self::client::VpMessageClientError;
pub use self::client::VpMessageClientErrorType;
pub use self::client::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
pub use self::error::DisclosureError;
pub use self::error::VpClientError;
pub use self::error::VpSessionError;
pub use self::error::VpVerifierError;
pub use self::session::VpDisclosureMissingAttributes;
pub use self::session::VpDisclosureProposal;
pub use self::session::VpDisclosureSession;
pub use self::uri_source::DisclosureUriSource;

mod client;
mod error;
mod session;
mod uri_source;

#[derive(Debug)]
pub enum DisclosureSessionState<M, P> {
    MissingAttributes(M),
    Proposal(P),
}

pub trait DisclosureSession<I, H = HttpVpMessageClient> {
    type MissingAttributes: DisclosureMissingAttributes;
    type Proposal: DisclosureProposal<I>;

    async fn start<S>(
        client: H,
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, VpSessionError>
    where
        S: MdocDataSource<MdocIdentifier = I>,
        H: VpMessageClient,
        Self: Sized;

    fn verifier_certificate(&self) -> &BorrowingCertificate;
    fn reader_registration(&self) -> &ReaderRegistration;
    fn session_state(&self) -> DisclosureSessionState<&Self::MissingAttributes, &Self::Proposal>;
    fn session_type(&self) -> SessionType;

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
}

pub trait DisclosureMissingAttributes {
    fn missing_attributes(&self) -> impl Iterator<Item = &AttributeIdentifier>;
}

pub trait DisclosureProposal<I> {
    fn proposed_source_identifiers<'a>(&'a self) -> impl Iterator<Item = &'a I>
    where
        I: 'a;
    fn proposed_attributes(&self) -> ProposedAttributes;

    async fn disclose<K, KF>(&self, key_factory: &KF) -> Result<Option<BaseUrl>, DisclosureError<VpSessionError>>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>;
}
