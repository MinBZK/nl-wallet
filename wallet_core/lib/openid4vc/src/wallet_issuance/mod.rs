pub mod authorization;
pub mod credential;
pub mod discovery;
pub mod issuance_session;
pub mod preview;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

use std::collections::HashSet;
use std::fmt::Debug;

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
use url::Url;
use utils::single_unique::MultipleItemsFound;
use wscd::wscd::IssuanceWscd;

use crate::CredentialErrorCode;
use crate::CredentialPreviewErrorCode;
use crate::ErrorResponse;
use crate::Format;
use crate::TokenErrorCode;
use crate::credential::Credential;
use crate::dpop::DpopError;
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

    #[error("unsupported credential format proposed for credential \"{0}\": {1}")]
    #[category(pd)]
    UnsupportedCredentialFormat(String, Format),

    #[error("different issuer registrations found in credential previews")]
    #[category(critical)]
    DifferentIssuerRegistrations(#[source] MultipleItemsFound),

    #[error("issuer has no credential configurations supported")]
    #[category(critical)]
    NoCredentialConfigurationsSupported,

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

    /// Parses the credential offer from the redirect URI, fetches issuer and OAuth metadata and then either returns an
    /// [`AuthorizationSession`] the caller can use to redirect the user into a web-based OAuth flow (if the Credential
    /// Offer contains an Authorization Code) or immediately returns an [`IssuanceSession`] that contains the caller
    /// can use to request issued credentials (if the Credential Offer contains a Pre-Authorized Code).
    async fn start_with_credential_offer(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError>;
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

    async fn reject_issuance(self) -> Result<(), WalletIssuanceError>;

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview];

    fn issuer_registration(&self) -> &IssuerRegistration;
}
