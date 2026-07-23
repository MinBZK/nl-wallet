pub mod authorization;
mod authorization_endpoints;
pub mod credential;
pub mod discovery;
pub mod issuance_session;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

use std::collections::HashMap;
use std::collections::HashSet;

use attestation_data::attributes::AttributesError;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayloadFromMdocError;
use attestation_data::credential_payload::CredentialPayloadFromSdJwtError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use crypto::trust_anchor::TrustAnchors;
use error_category::ErrorCategory;
use indexmap::IndexSet;
use itertools::Itertools;
use jwt::error::JwkConversionError;
use jwt::error::JwtParseError;
use jwt::error::JwtX5cVerifyError;
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
use wscd::wscd::WiaClient;

use self::authorization::OAuthError;
use self::authorization_endpoints::AuthorizationEndpointsError;
use self::credential::CredentialWithMetadata;
use self::issuance_session::IssuanceTypeMetadata;
use crate::credential::Credential;
use crate::dpop::DpopError;
use crate::errors::CredentialErrorCode;
use crate::errors::CredentialPreviewErrorCode;
use crate::errors::RemoteErrorResponse;
use crate::errors::TokenErrorCode;
use crate::issuer_identifier::IssuerIdentifier;
use crate::issuer_identifier::IssuerUrl;
use crate::jose::JwsAlgorithm;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::well_known::WellKnownError;
use crate::scope::Scope;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewError;

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

    #[error("JWT parse error: {0}")]
    JwtParse(#[from] JwtParseError),

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

    #[error(
        "no OAuth scope value present in Issuer Metadata for Credential Configuration ID(s): {}",
        .0.iter().join(", ")
    )]
    #[category(critical)]
    IssuerMetadataNoScope(Vec<CredentialConfigurationId>),

    #[error("could not push authorization request to server: {0:?}")]
    #[category(expected)]
    ParHttp(#[source] reqwest::Error),

    #[error("could not retrieve access token from issuer: {0:?}")]
    #[category(expected)]
    TokenRequestHttp(#[source] reqwest::Error),

    #[error("retrieving access token from issuer reported an error: {0:?}")]
    #[category(pd)]
    TokenRequest(Box<RemoteErrorResponse<TokenErrorCode>>),

    #[error("pre-authorized code is no longer valid: it has expired or was already used")]
    #[category(expected)]
    PreAuthorizedCodeExpired,

    #[error("could not retrieve credential preview from issuer: {0:?}")]
    #[category(expected)]
    CredentialPreviewHttp(#[source] reqwest::Error),

    #[error("retrieving credential preview from issuer reported an error: {0:?}")]
    #[category(pd)]
    CredentialPreview(Box<RemoteErrorResponse<CredentialPreviewErrorCode>>),

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
    CredentialRequest(Box<RemoteErrorResponse<CredentialErrorCode>>),

    #[error("could not reject credential(s) from issuer: {0:?}")]
    #[category(expected)]
    CredentialRejectionHttp(#[source] reqwest::Error),

    #[error("rejecting credential(s) from issuer reported an error: {0:?}")]
    #[category(pd)]
    CredentialRejection(Box<RemoteErrorResponse<CredentialErrorCode>>),

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

    #[error("unknown Credential Configuration ID(s) received in Token Response: {}", .0.iter().join(", "))]
    #[category(critical)]
    TokenResponseUnknownCredentialConfigIds(Vec<CredentialConfigurationId>),

    #[error("empty scope set received in Token Response")]
    #[category(critical)]
    TokenResponseEmptyScope,

    #[error("unknown scope values received in Token Response: {}", .0.iter().join(" "))]
    #[category(critical)]
    TokenResponseUnknownScope(Vec<Scope>),

    #[error("uri `{0}` has different host than issuer identifier `{1}`")]
    #[category(critical)]
    HostMismatchWithIssuerIdentifier(IssuerUrl, Box<IssuerIdentifier>),

    #[error("type metadata for `{0}` not found")]
    #[category(critical)]
    TypeMetadataNotFound(CredentialConfigurationId),

    #[error("could not read issuer registration from preview: {0}")]
    PreviewIssuerRegistration(#[source] CredentialPreviewError),

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

    #[error("not all authorization endpoints present: {0}")]
    AuthorizationEndpoints(#[source] AuthorizationEndpointsError),

    #[error("could not retrieve Credential Issuer metadata: {0}")]
    #[category(expected)]
    CredentialIssuerMetadataHttp(#[source] reqwest::Error),

    #[error(
        "issuer identifier in Credential Issuer metadata does not match, expected: {expected}, received: {received}"
    )]
    #[category(expected)]
    CredentialIssuerMetadataIdentifierMismatch {
        expected: Box<IssuerIdentifier>,
        received: Box<IssuerIdentifier>,
    },

    #[error("could not verify Credential Issuer metadata: {0}")]
    #[category(expected)]
    CredentialIssuerMetadataVerify(#[source] JwtX5cVerifyError),

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
    MdocCredentialPayload(#[from] CredentialPayloadFromMdocError),

    #[error("error converting SD-JWT to a CredentialPayload: {0}")]
    SdJwtCredentialPayloadError(#[from] CredentialPayloadFromSdJwtError),

    #[error("different issuers found in credential previews")]
    #[category(critical)]
    DifferentIssuers(#[source] MultipleItemsFound),

    #[error("missing query in credential offer URI")]
    #[category(critical)]
    MissingCredentialOfferQuery,

    #[error("failed to deserialize credential offer: {0}")]
    #[category(pd)]
    CredentialOfferDeserialization(#[source] serde_qs::Error),

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

    #[error("the Authorization Server does not support attestation-based client authentication")]
    #[category(expected)]
    NoAttestationBasedClientAuthSupport,

    #[error(
        "the Authorization Server does not support ES256 for client attestation signing: {}",
        .0.as_ref().map(|algs| algs.iter().join(", ")).unwrap_or_else(|| "<none>".to_string())
    )]
    #[category(expected)]
    ClientAttestationSigningAlgNotSupported(Option<IndexSet<JwsAlgorithm>>),

    #[error(
        "the Authorization Server does not support ES256 for client attestation PoP signing: {}",
        .0.as_ref().map(|algs| algs.iter().join(", ")).unwrap_or_else(|| "<none>".to_string())
    )]
    #[category(expected)]
    ClientAttestationPopSigningAlgNotSupported(Option<IndexSet<JwsAlgorithm>>),
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
    #[expect(
        clippy::too_many_arguments,
        reason = "helper method that calls either of two functions"
    )]
    async fn start(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        issuer_trust_anchors: &TrustAnchors,
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError>;

    /// Parses the Credential Offer from the redirect URI, fetches issuer and OAuth metadata and then returns an
    /// [`AuthorizationSession`] the caller can use to redirect the user into a web-based OAuth flow. If the credential
    /// offer contains a Pre-Authorized code, this returns an error.
    async fn start_authorization_code_flow(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Authorization, WalletIssuanceError>;

    /// Parses the Credential Offer from the redirect URI, fetches issuer and OAuth metadata and then returns an
    /// [`IssuanceSession`] that the caller can use to request issued credentials. If the Credential Offer resolves to
    /// an Authorization Code flow, this returns an error.
    async fn start_pre_authorized_code_flow(
        &self,
        offer_uri: &Url,
        issuer_trust_anchors: &TrustAnchors,
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
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
        wia_client: &impl WiaClient,
    ) -> Result<Self::Issuance, WalletIssuanceError>;
}

/// Represents an active credential issuance session for which previews are available.
pub trait IssuanceSession {
    async fn accept_issuance<W>(
        &mut self,
        trust_anchors: &TrustAnchors,
        wscd: &W,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd;

    async fn reject_issuance(&self) -> Result<(), WalletIssuanceError>;

    fn credential_previews(&self) -> &VecNonEmpty<CredentialPreview>;

    fn type_metadata(&self) -> &HashMap<CredentialConfigurationId, IssuanceTypeMetadata>;

    fn issuer_registration(&self) -> &IssuerRegistration;
}
