pub mod authorization;
pub mod credential;
pub mod discovery;
pub mod issuance_session;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

use std::collections::HashMap;
use std::collections::HashSet;

use attestation_data::attributes::AttributesError;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::MdocCredentialPayloadError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use crypto::trust_anchor::TrustAnchors;
use error_category::ErrorCategory;
use itertools::Itertools;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use mdoc::utils::cose::CoseError;
use reqwest::header::ToStrError;
use sd_jwt::error::DecoderError;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;
use utils::single_unique::MultipleItemsFound;
use utils::vec_at_least::VecNonEmpty;
use wscd::wscd::IssuanceWscd;

use crate::CredentialErrorCode;
use crate::CredentialPreviewErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;
use crate::credential::Credential;
use crate::dpop::DpopError;
use crate::issuer_identifier::IssuerIdentifier;
use crate::issuer_identifier::IssuerUrl;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::well_known::WellKnownError;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewError;
use crate::wallet_issuance::authorization::OAuthError;
use crate::wallet_issuance::credential::CredentialWithMetadata;
use crate::wallet_issuance::issuance_session::IssuanceTypeMetadata;

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

    #[error("could not push authorization request to server: {0:?}")]
    #[category(expected)]
    ParHttp(#[source] reqwest::Error),

    #[error("could not retrieve access token from issuer: {0:?}")]
    #[category(expected)]
    TokenRequestHttp(#[source] reqwest::Error),

    #[error("retrieving access token from issuer reported an error: {0:?}")]
    #[category(pd)]
    TokenRequest(Box<ErrorResponse<TokenErrorCode>>),

    #[error("could not retrieve credential preview from issuer: {0:?}")]
    #[category(expected)]
    CredentialPreviewHttp(#[source] reqwest::Error),

    #[error("retrieving credential preview from issuer reported an error: {0:?}")]
    #[category(pd)]
    CredentialPreview(Box<ErrorResponse<CredentialPreviewErrorCode>>),

    #[error("could not retrieve type metadata from issuer: {0:?}")]
    #[category(expected)]
    TypeMetadataHttp(#[source] reqwest::Error),

    #[error("could not retrieve nonce from issuer: {0:?}")]
    #[category(expected)]
    NonceHttp(#[source] reqwest::Error),

    #[error("could not retrieve credential(s) from issuer: {0:?}")]
    #[category(expected)]
    CredentialRequestHttp(#[source] reqwest::Error),

    #[error("retrieving credential(s) from issuer reported an error: {0:?}")]
    #[category(pd)]
    CredentialRequest(Box<ErrorResponse<CredentialErrorCode>>),

    #[error("could not reject credential(s) from issuer: {0:?}")]
    #[category(expected)]
    CredentialRejectionHttp(#[source] reqwest::Error),

    #[error("rejecting credential(s) from issuer reported an error: {0:?}")]
    #[category(pd)]
    CredentialRejection(Box<ErrorResponse<CredentialErrorCode>>),

    #[error("generating credential private keys failed: {0}")]
    #[category(pd)]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("WIA issuance failed: {0}")]
    #[category(pd)]
    WiaIssuance(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

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

    #[error("type metadata uri `{0}` does not start with issuer identifier `{1}`")]
    #[category(critical)]
    TypeMetadataUriNotBasedFromIssuerIdentifier(IssuerUrl, Box<IssuerIdentifier>),

    #[error("different attestation types found for same type metadata uri `{0}`")]
    #[category(critical)]
    DifferentAttestationTypeForSameTypeMetadataUri(IssuerUrl),

    #[error("different type metadata found for attestation type `{0}`")]
    #[category(critical)]
    DifferentTypeMetadataForSameAttestationType(String),

    #[error("type metadata for `{0}` not found")]
    #[category(critical)]
    TypeMetadataNotFound(String),

    #[error("error verifying credential preview: {0}")]
    CredentialPreviewVerification(#[source] CredentialPreviewError),

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

    #[error(
        "authorization server specified in Credential Offer is not present in OAuth metadata: {} not in {}",
        .0,
        .1.iter().join(" or ")
    )]
    #[category(expected)]
    AuthorizationServerMismatch(Box<IssuerIdentifier>, Box<VecNonEmpty<IssuerIdentifier>>),

    #[error("missing Credential Configuration ID from Credential Offer in Issuer Metadata: {}", .0.iter().join(", "))]
    #[category(expected)]
    MissingCredentialConfigId(HashSet<CredentialConfigurationId>),

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

    #[error("malformed attribute: random too short (was {0}; minimum {1}")]
    #[category(critical)]
    AttributeRandomLength(usize, usize),

    #[error("error converting mdoc to a CredentialPayload: {0}")]
    MdocCredentialPayload(#[from] MdocCredentialPayloadError),

    #[error("error converting SD-JWT to a CredentialPayload: {0}")]
    SdJwtCredentialPayloadError(#[from] SdJwtCredentialPayloadError),

    #[error("different issuer registrations found in credential previews")]
    #[category(critical)]
    DifferentIssuerRegistrations(#[source] MultipleItemsFound),

    #[error("missing query in credential offer URI")]
    #[category(critical)]
    MissingCredentialOfferQuery,

    #[error("failed to deserialize credential offer: {0}")]
    #[category(pd)]
    CredentialOfferDeserialization(#[source] serde_urlencoded::de::Error),

    #[error("could not retrieve credential offer from issuer: {0:?}")]
    #[category(expected)]
    CredentialOfferHttp(#[source] reqwest::Error),

    #[error("only unknown grant type(s) found in Credential Offer: {}", .0.iter().join(", "))]
    #[category(expected)]
    CredentialOfferUnknownGrants(HashSet<String>),

    #[error(
        "the Credential Offer did not contain any grants and the Authorization Server does not support the \
         Authorization Code grant"
    )]
    #[category(expected)]
    AuthorizationCodeNotSupported,

    #[error("a Credential Offer containing a Pre-Authorized code with a Transaction Code is unsupported")]
    #[category(expected)]
    CredentialOfferTxCodeUnsupported,

    #[error("the Credential Offer did not resolve to using the Authorization Code flow")]
    #[category(expected)]
    CredentialOfferNoAuthorizationCode,

    #[error("the Credential Offer did not contain a Pre-Authorized Code")]
    #[category(expected)]
    CredentialOfferNoPreAuthorizedCode,
}

#[derive(Debug)]
pub enum IssuanceFlow<A, I> {
    AuthorizationCode { authorization_session: A },
    PreAuthorizedCode { issuance_session: I },
}

/// Discovers credential issuer and OAuth authorization server metadata, then starts an issuance flow.
pub trait IssuanceDiscovery {
    type Authorization: AuthorizationSession<Issuance = Self::Issuance>;
    type Issuance: IssuanceSession;

    /// Parses the Credential Offer from the redirect URI, fetches issuer and OAuth metadata and then either returns an
    /// [`AuthorizationSession`] the caller can use to redirect the user into a web-based OAuth flow (if the Credential
    /// Offer resolves to an Authorization Code flow) or immediately returns an [`IssuanceSession`] that the caller can
    /// use to request issued credentials (if the Credential Offer contains a Pre-Authorized Code).
    async fn start(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError>;

    /// Parses the Credential Offer from the redirect URI, fetches issuer and OAuth metadata and then returns an
    /// [`AuthorizationSession`] the caller can use to redirect the user into a web-based OAuth flow. If the credential
    /// offer contains a Pre-Authorized code, this returns an error.
    async fn start_authorization_code_flow(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError>;

    /// Parses the Credential Offer from the redirect URI, fetches issuer and OAuth metadata and then returns an
    /// [`IssuanceSession`] that the caller can use to request issued credentials. If the Credential Offer resolves to
    /// an Authorization Code flow, this returns an error.
    async fn start_pre_authorized_code_flow(
        &self,
        offer_uri: &Url,
        client_id: String,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError>;

    /// Rebuilds an [`AuthorizationSession`] from data that was persisted before the app left memory.
    fn restore_authorization_session(
        &self,
        data: <Self::Authorization as AuthorizationSession>::Persisted,
    ) -> Self::Authorization;
}

/// Represents an in-progress OAuth authorization code flow.
///
/// The caller should redirect the user to [`auth_url`](AuthorizationSession::auth_url) and then
/// call [`start_issuance`](AuthorizationSession::start_issuance) with the redirect URI the
/// authorization server returns after the user authenticates.
pub trait AuthorizationSession {
    type Issuance: IssuanceSession;
    type Persisted: Clone + Send + Sync + Serialize + DeserializeOwned + 'static;

    /// Returns the authorization URL the user should be redirected to.
    fn auth_url(&self) -> &Url;

    /// Returns the OAuth `state` (CSRF token) stored in the PAR-submitted authorization request.
    fn state(&self) -> &str;

    /// Returns the data needed to resume the authorization code flow after an app restart.
    fn persist(&self) -> Self::Persisted;

    /// Exchanges the authorization code in `received_redirect_uri` for an access token and
    /// credential previews, returning an [`IssuanceSession`].
    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError>;
}

/// Represents an active credential issuance session for which previews are available.
pub trait IssuanceSession {
    async fn accept_issuance<W>(
        &mut self,
        trust_anchors: &TrustAnchors,
        wscd: &W,
        include_wia: bool,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd;

    async fn reject_issuance(&self) -> Result<(), WalletIssuanceError>;

    fn credential_previews(&self) -> &[CredentialPreview];

    fn type_metadata(&self) -> &HashMap<String, IssuanceTypeMetadata>;

    fn issuer_registration(&self) -> &IssuerRegistration;
}
