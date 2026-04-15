pub mod authorization;
pub mod credential;
pub mod discovery;
pub mod issuance_session;
pub mod preview;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

use std::collections::HashSet;
use std::fmt::Debug;

use itertools::Itertools;
use reqwest::header::ToStrError;
use rustls_pki_types::TrustAnchor;
use url::Url;

use attestation_data::attributes::AttributesError;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::MdocCredentialPayloadError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use error_category::ErrorCategory;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use mdoc::utils::cose::CoseError;
use sd_jwt::error::DecoderError;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use utils::single_unique::MultipleItemsFound;
use wscd::Poa;
use wscd::wscd::IssuanceWscd;

use crate::CredentialErrorCode;
use crate::CredentialPreviewErrorCode;
use crate::ErrorResponse;
use crate::Format;
use crate::TokenErrorCode;
use crate::credential::Credential;
use crate::dpop::DpopError;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::well_known::WellKnownError;
use crate::token::CredentialPreviewError;
use crate::wallet_issuance::authorization::OAuthError;
use crate::wallet_issuance::credential::CredentialWithMetadata;
use crate::wallet_issuance::preview::NormalizedCredentialPreview;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletIssuanceError {
    #[error("failed to get public key: {0}")]
    #[category(pd)]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("DPoP error: {0}")]
    Dpop(#[from] DpopError),

    #[error("failed to convert key from/to JWK format: {0}")]
    JwkConversion(#[from] JwkConversionError),

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),

    #[error("http request failed: {0}")]
    #[category(expected)]
    Network(#[from] reqwest::Error),

    #[error("missing c_nonce")]
    #[category(critical)]
    MissingNonce,

    #[error("mismatch between issued and previewed credential, issued: {actual:?} , previewed: {expected:?}")]
    #[category(pd)]
    IssuedCredentialMismatch {
        actual: Box<PreviewableCredentialPayload>,
        expected: Box<PreviewableCredentialPayload>,
    },

    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] mdoc::Error),

    #[error("SD-JWT verification failed: {0}")]
    #[category(pd)]
    SdJwtVerification(#[from] DecoderError),

    #[error("type metadata verification failed: {0}")]
    #[category(critical)]
    TypeMetadataVerification(#[from] TypeMetadataChainError),

    #[error("attributes do not match type metadata: {0}")]
    #[category(pd)]
    AttributesVerification(#[from] AttributesError),

    #[error("error requesting access token: {0:?}")]
    #[category(pd)]
    TokenRequest(Box<ErrorResponse<TokenErrorCode>>),

    #[error("error requesting credentials: {0:?}")]
    #[category(pd)]
    CredentialRequest(Box<ErrorResponse<CredentialErrorCode>>),

    #[error("generating credential private keys failed: {0}")]
    #[category(pd)]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("public key contained in mdoc not equal to expected value")]
    #[category(critical)]
    PublicKeyMismatch,

    #[error("received {found} responses, expected {expected}")]
    #[category(critical)]
    UnexpectedCredentialResponseCount { found: usize, expected: usize },

    #[error("deferred issuance is not supported")]
    #[category(expected)]
    DeferredIssuanceUnsupported,

    #[error("received credential response: {actual:?}, expected type {expected}")]
    #[category(pd)]
    UnexpectedCredentialResponseType { expected: String, actual: Credential },

    #[error("error reading HTTP error: {0}")]
    #[category(pd)]
    HeaderToStr(#[from] ToStrError),

    #[error("error verifying credential preview: {0}")]
    CredentialPreview(#[from] CredentialPreviewError),

    #[error("error retrieving issuer certificate from issued mdoc: {0}")]
    IssuerCertificate(#[source] CoseError),

    #[error("issuer contained in credential not equal to expected value")]
    #[category(critical)]
    IssuerMismatch,

    #[error("error retrieving metadata from issued mdoc: {0}")]
    Metadata(#[source] mdoc::Error),

    #[error("metadata integrity digest contained is not consistent across credential copies")]
    #[category(critical)]
    MetadataIntegrityInconsistent,

    #[error("missing metadata integrity digest")]
    #[category(critical)]
    MetadataIntegrityMissing,

    #[error("error discovering Oauth metadata: {0}")]
    #[category(expected)]
    OauthDiscovery(#[source] WellKnownError),

    #[error("error discovering Credential Issuer metadata: {0}")]
    #[category(expected)]
    CredentialIssuerDiscovery(#[source] WellKnownError),

    #[error("error during OAuth: {0}")]
    #[category(expected)]
    OAuth(#[from] OAuthError),

    #[error("issuer has no batch credential endpoint")]
    #[category(critical)]
    NoBatchCredentialEndpoint,

    #[error("issuer has no credential preview endpoint")]
    #[category(critical)]
    NoCredentialPreviewEndpoint, // TODO (PVW-5559): skip preview when no credential preview endpoint

    #[error("issuer has no nonce endpoint, yet one of the credential configurations require cryptographic binding")]
    #[category(critical)]
    NoNonceEndpoint,

    #[error("error requesting credential preview: {0:?}")]
    #[category(pd)]
    CredentialPreviewRequest(Box<ErrorResponse<CredentialPreviewErrorCode>>),

    #[error("malformed attribute: random too short (was {0}; minimum {1}")]
    #[category(critical)]
    AttributeRandomLength(usize, usize),

    #[error("error converting mdoc to a CredentialPayload: {0}")]
    MdocCredentialPayload(#[from] MdocCredentialPayloadError),

    #[error("error converting SD-JWT to a CredentialPayload: {0}")]
    SdJwtCredentialPayloadError(#[from] SdJwtCredentialPayloadError),

    #[error("unsupported credential format(s) proposed for credential \"{}\": {}", .0, .1.iter().join(", "))]
    #[category(pd)]
    UnsupportedCredentialFormat(String, HashSet<Format>),

    #[error("different issuer registrations found in credential previews")]
    #[category(critical)]
    DifferentIssuerRegistrations(#[source] MultipleItemsFound),

    #[error("issuer has no credential configurations supported")]
    #[category(critical)]
    NoCredentialConfigurationsSupported,

    #[error("no Authorization Code found in Credential Offer")]
    #[category(critical)]
    MissingPreAuthorizedCodeGrant,

    #[error("missing query in credential offer URI")]
    #[category(critical)]
    MissingCredentialOfferQuery,

    #[error("failed to deserialize credential offer: {0}")]
    #[category(pd)]
    CredentialOfferDeserialization(#[source] serde_urlencoded::de::Error),
}

/// Discovers credential issuer and OAuth authorization server metadata, then starts an issuance flow.
pub trait IssuanceDiscovery {
    type Authorization: AuthorizationSession<Issuance = Self::Issuance>;
    type Issuance: IssuanceSession;

    /// Fetches issuer and OAuth metadata, constructs a PKCE-protected authorization URL, and returns
    /// an [`AuthorizationSession`] the caller can use to redirect the user and later exchange the
    /// authorization code for credentials.
    async fn start_authorization_code_flow(
        &self,
        identifier: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError>;

    /// Parses the credential offer from the redirect URI, fetches issuer and OAuth metadata,
    /// exchanges the pre-authorized code for an access token, and returns an [`IssuanceSession`]
    /// ready to request credentials.
    async fn start_pre_authorized_code_flow(
        &self,
        redirect_uri: &Url,
        client_id: String,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError>;
}

/// Represents an in-progress OAuth authorization code flow.
///
/// The caller should redirect the user to [`auth_url`](AuthorizationSession::auth_url) and then
/// call [`start_issuance`](AuthorizationSession::start_issuance) with the redirect URI the
/// authorization server returns after the user authenticates.
pub trait AuthorizationSession {
    type Issuance: IssuanceSession;

    /// Returns the authorization URL the user should be redirected to.
    fn auth_url(&self) -> &Url;

    /// Exchanges the authorization code in `received_redirect_uri` for an access token and
    /// credential previews, returning an [`IssuanceSession`].
    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError>;
}

/// Represents an active credential issuance session for which previews are available.
pub trait IssuanceSession {
    async fn accept_issuance<W>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        wscd: &W,
        include_wua: bool,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd<Poa = Poa>;

    async fn reject_issuance(self) -> Result<(), WalletIssuanceError>;

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview];

    fn issuer_registration(&self) -> &IssuerRegistration;
}
