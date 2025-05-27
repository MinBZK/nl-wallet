use std::collections::HashSet;
use std::collections::VecDeque;
use std::hash::Hash;

use derive_more::Debug;
use futures::future::try_join_all;
use futures::future::OptionFuture;
use futures::TryFutureExt;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::rand_core::OsRng;
use reqwest::header::ToStrError;
use reqwest::header::AUTHORIZATION;
use reqwest::Method;
use rustls_pki_types::TrustAnchor;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::IntoCredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use crypto::factory::KeyFactory;
use crypto::keys::CredentialEcdsaKey;
use crypto::x509::BorrowingCertificate;
use error_category::ErrorCategory;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredential;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::jwk::jwk_to_p256;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteClaims;
use jwt::EcdsaDecodingKey;
use jwt::Jwt;
use mdoc::holder::Mdoc;
use mdoc::holder::MdocCredentialPayloadError;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborBase64;
use mdoc::utils::serialization::TaggedBytes;
use mdoc::ATTR_RANDOM_LENGTH;
use poa::factory::PoaFactory;
use poa::Poa;
use sd_jwt::hasher::Sha256Hasher;
use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use utils::generator::TimeGenerator;
use utils::single_unique::MultipleItemsFound;
use utils::single_unique::SingleUnique;
use utils::vec_at_least::VecAtLeastTwoUnique;
use utils::vec_at_least::VecNonEmpty;

use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequestType;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::credential::WteDisclosure;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::dpop::DPOP_HEADER_NAME;
use crate::dpop::DPOP_NONCE_HEADER_NAME;
use crate::metadata::IssuerMetadata;
use crate::oidc;
use crate::token::AccessToken;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewContent;
use crate::token::CredentialPreviewError;
use crate::token::TokenRequest;
use crate::token::TokenResponseWithPreviews;
use crate::CredentialErrorCode;
use crate::ErrorResponse;
use crate::Format;
use crate::TokenErrorCode;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum IssuanceSessionError {
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

    #[error("missing issuer certificate")]
    #[category(critical)]
    MissingIssuerCertificate,

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
    SdJwtVerification(#[from] sd_jwt::error::Error),

    #[error("type metadata verification failed: {0}")]
    #[category(critical)]
    TypeMetadataVerification(#[from] TypeMetadataChainError),

    #[error("error requesting access token: {0:?}")]
    #[category(pd)]
    TokenRequest(ErrorResponse<TokenErrorCode>),

    #[error("error requesting credentials: {0:?}")]
    #[category(pd)]
    CredentialRequest(ErrorResponse<CredentialErrorCode>),

    #[error("generating credential private keys failed: {0}")]
    #[category(pd)]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("public key contained in mdoc not equal to expected value")]
    #[category(critical)]
    PublicKeyMismatch,

    #[error("received {found} responses, expected {expected}")]
    #[category(critical)]
    UnexpectedCredentialResponseCount { found: usize, expected: usize },

    #[error("received credential response: {actual:?}, expected type {expected}")]
    #[category(pd)]
    UnexpectedCredentialResponseType {
        expected: String,
        actual: CredentialResponse,
    },

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

    #[error("metadata contained in credential not equal to expected value")]
    #[category(critical)]
    MetadataMismatch,

    #[error("metadata integrity digest contained is not consistent across credential copies")]
    #[category(critical)]
    MetadataIntegrityInconsistent,

    #[error("missing metadata integrity digest")]
    #[category(critical)]
    MetadataIntegrityMissing,

    #[error("error discovering Oauth metadata: {0}")]
    #[category(expected)]
    OauthDiscovery(#[source] reqwest::Error),

    #[error("error discovering OpenID4VCI Credential Issuer metadata: {0}")]
    #[category(expected)]
    OpenId4vciDiscovery(#[source] reqwest::Error),

    #[error("issuer has no batch credential endpoint")]
    #[category(critical)]
    NoBatchCredentialEndpoint,

    #[error("malformed attribute: random too short (was {0}; minimum {1}")]
    #[category(critical)]
    AttributeRandomLength(usize, usize),

    #[error("error constructing PoA: {0}")]
    #[category(pd)]
    Poa(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

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
}

#[derive(Clone, Debug)]
pub enum IssuedCredential {
    MsoMdoc(Box<Mdoc>),
    SdJwt(Box<SdJwt>),
}

#[derive(Clone, Debug)]
pub enum IssuedCredentialCopies {
    MsoMdoc(VecNonEmpty<Mdoc>),
    SdJwt(VecNonEmpty<SdJwt>),
}

impl IssuedCredentialCopies {
    pub fn len(&self) -> usize {
        match self {
            IssuedCredentialCopies::MsoMdoc(mdocs) => mdocs.len().into(),
            IssuedCredentialCopies::SdJwt(sdjwts) => sdjwts.len().into(),
        }
    }

    // Required by clippy
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, IssuanceSessionError> {
        match self {
            IssuedCredentialCopies::MsoMdoc(mdocs) => mdocs
                .first()
                .issuer_certificate()
                .map_err(mdoc::Error::Cose)
                .map_err(IssuanceSessionError::MdocVerification),
            IssuedCredentialCopies::SdJwt(sd_jwts) => {
                let cert = sd_jwts
                    .first()
                    .issuer_certificate()
                    .ok_or(IssuanceSessionError::MissingIssuerCertificate)?;
                Ok(cert.clone())
            }
        }
    }
}

pub trait IssuanceSession<H = HttpVcMessageClient> {
    async fn start_issuance(
        message_client: H,
        base_url: BaseUrl,
        token_request: TokenRequest,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, IssuanceSessionError>
    where
        Self: Sized;

    async fn accept_issuance<K, KF>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &KF,
        wte: Option<JwtCredential<WteClaims>>,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>;

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError>;

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview];

    fn issuer_registration(&self) -> &IssuerRegistration;
}

#[derive(Debug)]
pub struct HttpIssuanceSession<H = HttpVcMessageClient> {
    message_client: H,
    session_state: IssuanceState,
}

/// Contract for sending OpenID4VCI protocol messages.
#[cfg_attr(test, mockall::automock)]
pub trait VcMessageClient {
    fn client_id(&self) -> &str;
    async fn discover_metadata(&self, url: &BaseUrl) -> Result<IssuerMetadata, IssuanceSessionError>;
    async fn discover_oauth_metadata(&self, url: &BaseUrl) -> Result<oidc::Config, IssuanceSessionError>;

    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuanceSessionError>;

    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, IssuanceSessionError>;

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuanceSessionError>;

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str)
        -> Result<(), IssuanceSessionError>;
}

pub struct HttpVcMessageClient {
    client_id: String,
    http_client: reqwest::Client,
}

impl HttpVcMessageClient {
    pub fn new(client_id: String, http_client: reqwest::Client) -> Self {
        Self { client_id, http_client }
    }
}

impl VcMessageClient for HttpVcMessageClient {
    fn client_id(&self) -> &str {
        &self.client_id
    }

    async fn discover_metadata(&self, url: &BaseUrl) -> Result<IssuerMetadata, IssuanceSessionError> {
        let metadata = IssuerMetadata::discover(&self.http_client, url)
            .await
            .map_err(IssuanceSessionError::OpenId4vciDiscovery)?;
        Ok(metadata)
    }

    async fn discover_oauth_metadata(&self, url: &BaseUrl) -> Result<oidc::Config, IssuanceSessionError> {
        let metadata = self
            .http_client
            .get(url.join("/.well-known/oauth-authorization-server"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(IssuanceSessionError::OauthDiscovery)?;
        Ok(metadata)
    }

    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuanceSessionError> {
        self.http_client
            .post(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header.as_ref())
            .form(&token_request)
            .send()
            .map_err(IssuanceSessionError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorCode>>().await?;
                    Err(IssuanceSessionError::TokenRequest(error))
                } else {
                    let dpop_nonce = response
                        .headers()
                        .get(DPOP_NONCE_HEADER_NAME)
                        .map(|val| val.to_str())
                        .transpose()?
                        .map(str::to_string);
                    let deserialized = response.json::<TokenResponseWithPreviews>().await?;
                    Ok((deserialized, dpop_nonce))
                }
            })
            .await
    }

    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, IssuanceSessionError> {
        self.request(url, credential_request, dpop_header, access_token_header)
            .await
    }

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuanceSessionError> {
        self.request(url, credential_requests, dpop_header, access_token_header)
            .await
    }

    async fn reject(
        &self,
        url: &Url,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<(), IssuanceSessionError> {
        self.http_client
            .delete(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .send()
            .map_err(IssuanceSessionError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorCode>>().await?;
                    Err(IssuanceSessionError::CredentialRequest(error))
                } else {
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }
}

impl HttpVcMessageClient {
    async fn request<T: Serialize, S: DeserializeOwned>(
        &self,
        url: &Url,
        request: &T,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<S, IssuanceSessionError> {
        self.http_client
            .post(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .json(request)
            .send()
            .map_err(IssuanceSessionError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorCode>>().await?;
                    Err(IssuanceSessionError::CredentialRequest(error))
                } else {
                    let response = response.json().await?;
                    Ok(response)
                }
            })
            .await
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedCredentialPreview {
    pub content: CredentialPreviewContent,

    pub normalized_metadata: NormalizedTypeMetadata,

    pub raw_metadata: SortedTypeMetadataDocuments,
}

impl NormalizedCredentialPreview {
    pub fn try_new(preview: CredentialPreview) -> Result<Self, IssuanceSessionError> {
        let (normalized_metadata, raw_metadata) = preview
            .type_metadata
            .into_normalized(&preview.content.credential_payload.attestation_type)?;

        Ok(Self {
            content: preview.content,
            normalized_metadata,
            raw_metadata,
        })
    }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    c_nonce: String,
    normalized_credential_previews: Vec<NormalizedCredentialPreview>,
    credential_request_types: Vec<CredentialRequestType>,
    issuer_registration: IssuerRegistration,
    issuer_url: BaseUrl,
    #[debug(skip)]
    dpop_private_key: SigningKey,
    dpop_nonce: Option<String>,
}

fn credential_request_types_from_preview(
    normalized_credential_previews: &[NormalizedCredentialPreview],
) -> Result<Vec<CredentialRequestType>, IssuanceSessionError> {
    // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
    // has no support (yet) for issuance of multiple copies of multiple attestations.
    // We implement this below by simply flattening the relevant nested iterators when communicating with the
    // issuer.
    //
    // The `/batch_credential` endpoint also does not support reading the `CredentialRequest::credential_type`
    // field, it will simply provide the wallet with copies of all of the credential formats it proposes.
    // For this reason, it is simply an error if the wallet does not support all of the formats proposed by
    // the issuer.
    //
    // TODO (PVW-4366): Have the batch issuance endpoint consider the `credential_type` field
    //                  of the `CredentialRequest`s and only issue those formats.

    let credential_request_types = normalized_credential_previews
        .iter()
        .map(|preview| {
            let mut unsupported_formats = HashSet::new();

            // Construct a `Vec<CredentialRequestType>`, with one entry
            // per copy per supported format for this credential.
            let request_types = preview
                .content
                .copies_per_format
                .iter()
                .flat_map(|(format, copies)| {
                    let request_type = CredentialRequestType::from_format(
                        *format,
                        preview.content.credential_payload.attestation_type.clone(),
                    );

                    if request_type.is_none() {
                        unsupported_formats.insert(*format);
                    }

                    request_type.map(|request_type| itertools::repeat_n(request_type, copies.get().into()))
                })
                .flatten()
                .collect_vec();

            // If we do not support one of the proposed formats this constitutes an error, as described above.
            if !unsupported_formats.is_empty() {
                return Err(IssuanceSessionError::UnsupportedCredentialFormat(
                    preview.content.credential_payload.attestation_type.clone(),
                    unsupported_formats,
                ));
            }

            Ok(request_types)
        })
        .process_results(|iter| iter.flatten().collect_vec())?;

    Ok(credential_request_types)
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    /// Discover the token endpoint from the OAuth server metadata.
    async fn discover_token_endpoint(message_client: &H, base_url: &BaseUrl) -> Result<Url, IssuanceSessionError> {
        let issuer_metadata = message_client.discover_metadata(base_url).await?;

        // The issuer may announce multiple OAuth authorization servers the wallet may use. Which one the wallet
        // uses is left up to the wallet. We just take the first one.
        // authorization_servers() always returns a non-empty vec so the unwrap() is safe.
        let authorization_servers = &issuer_metadata.issuer_config.authorization_servers();
        let oauth_server = authorization_servers.first().unwrap();
        let oauth_metadata = message_client.discover_oauth_metadata(oauth_server).await?;

        let token_endpoint = oauth_metadata.token_endpoint.clone();
        Ok(token_endpoint)
    }

    /// Discover the credential endpoint from the Credential Issuer metadata.
    async fn discover_credential_endpoint(message_client: &H, base_url: &BaseUrl) -> Result<Url, IssuanceSessionError> {
        let url = message_client
            .discover_metadata(base_url)
            .await?
            .issuer_config
            .credential_endpoint
            .as_ref()
            .clone();

        Ok(url)
    }

    /// Discover the batch credential endpoint from the Credential Issuer metadata.
    /// This function returns an `Option` because the batch credential is optional.
    async fn discover_batch_credential_endpoint(
        message_client: &H,
        base_url: &BaseUrl,
    ) -> Result<Option<Url>, IssuanceSessionError> {
        let url = message_client
            .discover_metadata(base_url)
            .await?
            .issuer_config
            .batch_credential_endpoint
            .map(|url| url.as_ref().clone());
        Ok(url)
    }
}

impl<H: VcMessageClient> IssuanceSession<H> for HttpIssuanceSession<H> {
    async fn start_issuance(
        message_client: H,
        base_url: BaseUrl,
        token_request: TokenRequest,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, IssuanceSessionError> {
        let token_endpoint = Self::discover_token_endpoint(&message_client, &base_url).await?;

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, token_endpoint.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = message_client
            .request_token(&token_endpoint, &token_request, &dpop_header)
            .await?;

        let issuer_registration = token_response
            .credential_previews
            .iter()
            .map(|preview| preview.issuer_registration())
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .single_unique()
            .map_err(IssuanceSessionError::DifferentIssuerRegistrations)?
            .expect("there are always credential_previews in the token_response");

        let normalized_credential_previews = token_response
            .credential_previews
            .into_iter()
            .map(|preview| {
                // Verify the issuer certificate against the trust anchors.
                preview.verify(trust_anchors)?;
                let state = NormalizedCredentialPreview::try_new(preview)?;
                Ok::<_, IssuanceSessionError>(state)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let credential_request_types = credential_request_types_from_preview(&normalized_credential_previews)?;

        let session_state = IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response
                .token_response
                .c_nonce
                .ok_or(IssuanceSessionError::MissingNonce)?,
            normalized_credential_previews,
            credential_request_types,
            issuer_registration,
            issuer_url: base_url,
            dpop_private_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };

        Ok(issuance_client)
    }

    async fn accept_issuance<K, KF>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &KF,
        wte: Option<JwtCredential<WteClaims>>,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
    {
        // Generate the PoPs to be sent to the issuer, and the private keys with which they were generated
        // (i.e., the private key of the future mdoc).
        // If N is the total amount of copies of credentials to be issued, then this returns N key/proof pairs.
        // Note that N > 0 because `self.session_state.credential_request_types`` which we mapped above is
        // derived from `VecNonEmpty<_>`.
        let keys_and_proofs = CredentialRequestProof::new_multiple(
            self.session_state.c_nonce.clone(),
            self.message_client.client_id().to_string(),
            self.session_state.issuer_url.clone(),
            self.session_state.credential_request_types.len().try_into().unwrap(),
            key_factory,
        )
        .await?;

        let pop_claims = JwtPopClaims::new(
            Some(self.session_state.c_nonce.clone()),
            self.message_client.client_id().to_string(),
            self.session_state.issuer_url.as_ref().to_string(),
        );

        // This could be written better with `Option::map`, but `Option::map` does not support async closures
        let (mut wte_disclosure, wte_privkey) = match wte {
            Some(wte) => {
                let wte_privkey = wte.private_key(key_factory)?;
                let wte_release =
                    Jwt::<JwtPopClaims>::sign(&pop_claims, &Header::new(Algorithm::ES256), &wte_privkey).await?;
                (Some(WteDisclosure::new(wte.jwt, wte_release)), Some(wte_privkey))
            }
            None => (None, None),
        };

        // Ensure we include the WTE private key in the keys we need to prove association for.
        let poa_keys = keys_and_proofs
            .iter()
            .map(|(key, _)| key)
            .chain(wte_privkey.as_ref())
            .collect_vec();

        // We need a minimum of two keys to associate for a PoA to be sensible.
        let poa = VecAtLeastTwoUnique::try_from(poa_keys).ok().map(|poa_keys| async {
            key_factory
                .poa(poa_keys, pop_claims.aud.clone(), pop_claims.nonce.clone())
                .await
                .map_err(|e| IssuanceSessionError::Poa(Box::new(e)))
        });
        let mut poa = OptionFuture::from(poa).await.transpose()?;

        // Split into N keys and N credential requests, so we can send the credential request proofs separately
        // to the issuer.
        let (pubkeys, credential_requests): (Vec<_>, Vec<_>) = try_join_all(
            keys_and_proofs
                .into_iter()
                .zip(self.session_state.credential_request_types.clone())
                .map(|((key, proof), credential_request_type)| async move {
                    let pubkey = key
                        .verifying_key()
                        .await
                        .map_err(|e| IssuanceSessionError::VerifyingKeyFromPrivateKey(e.into()))?;
                    let id = key.identifier().to_string();
                    let cred_request = CredentialRequest {
                        credential_type: credential_request_type.into(),
                        proof: Some(proof),
                        attestations: None, // We set this field below if necessary
                        poa: None,          // We set this field below if necessary
                    };
                    Ok::<_, IssuanceSessionError>(((pubkey, id), cred_request))
                }),
        )
        .await?
        .into_iter()
        .unzip();

        // The following two unwraps are safe because N > 0, see above.
        let mut credential_requests = credential_requests; // Make it mutable so we can pop() to avoid cloning
        let responses = match credential_requests.len() {
            1 => {
                let mut credential_request = credential_requests.pop().unwrap();
                credential_request.attestations = wte_disclosure.take();
                credential_request.poa = poa.take();
                vec![self.request_credential(&credential_request).await?]
            }
            _ => {
                let credential_requests = VecNonEmpty::try_from(credential_requests).unwrap();
                self.request_batch_credentials(credential_requests, wte_disclosure.take(), poa.take())
                    .await?
            }
        };
        let mut responses_and_pubkeys: VecDeque<_> = responses.into_iter().zip(pubkeys).collect();

        let docs = self
            .session_state
            .normalized_credential_previews
            .iter()
            .map(|preview| {
                preview
                    .content
                    .copies_per_format
                    .iter()
                    .map(|(format, copies)| {
                        let copy_count: usize = copies.get().into();

                        // Consume the amount of copies from the front of `responses_and_keys`.
                        let cred_copies = VecNonEmpty::try_from(
                            responses_and_pubkeys
                                .drain(..copy_count)
                                .map(|(cred_response, (pubkey, key_id))| {
                                    if !cred_response.matches_format(*format) {
                                        return Err(IssuanceSessionError::UnexpectedCredentialResponseType {
                                            expected: format.to_string(),
                                            actual: cred_response.clone(),
                                        });
                                    }

                                    // Convert the response into a credential, verifying it against both the
                                    // trust anchors and the credential preview we received in the preview.
                                    cred_response.into_credential::<K>(key_id, &pubkey, preview, trust_anchors)
                                })
                                .collect::<Result<Vec<IssuedCredential>, _>>()?,
                        )
                        .expect("the resulting vector is never empty since 'copies' is nonzero");

                        // Verify that each of the resulting mdocs contain exactly the same metadata integrity digest.
                        let integrity = cred_copies
                            .iter()
                            .map(|cred_copy| match cred_copy {
                                IssuedCredential::MsoMdoc(mdoc) => {
                                    mdoc.type_metadata_integrity().map_err(IssuanceSessionError::Metadata)
                                }
                                IssuedCredential::SdJwt(sd_jwt) => sd_jwt
                                    .claims()
                                    .vct_integrity
                                    .as_ref()
                                    .ok_or(IssuanceSessionError::MetadataIntegrityMissing),
                            })
                            .process_results(|iter| {
                                iter.dedup()
                                    .exactly_one()
                                    .map_err(|_| IssuanceSessionError::MetadataIntegrityInconsistent)
                            })??;

                        // Check that the integrity hash received in the credential matches
                        // that of encoded JSON of the first metadata document.
                        preview.raw_metadata.verify(integrity.clone())?;

                        let copies = match format {
                            Format::MsoMdoc => IssuedCredentialCopies::MsoMdoc(
                                VecNonEmpty::try_from(
                                    cred_copies
                                        .into_inner()
                                        .into_iter()
                                        .map(|credential| match credential {
                                            IssuedCredential::MsoMdoc(mdoc) => *mdoc,
                                            _ => panic!("format and responses have already been verified"),
                                        })
                                        .collect_vec(),
                                )
                                .unwrap(), // This safe since `cred_copies` is never empty
                            ),
                            Format::SdJwt => IssuedCredentialCopies::SdJwt(
                                VecNonEmpty::try_from(
                                    cred_copies
                                        .into_inner()
                                        .into_iter()
                                        .map(|credential| match credential {
                                            IssuedCredential::SdJwt(sd_jwt) => *sd_jwt,
                                            _ => panic!("format and responses have already been verified"),
                                        })
                                        .collect_vec(),
                                )
                                .unwrap(), // This safe since `cred_copies` is never empty
                            ),
                            other => Err(IssuanceSessionError::UnsupportedCredentialFormat(
                                preview.content.credential_payload.attestation_type.clone(),
                                HashSet::from([*other]),
                            ))?,
                        };

                        Ok(copies)
                    })
                    .collect::<Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>>()
            })
            // Flatten the results, s.t. we're left with a mixed vector of IssuedCredentialCopies
            .process_results(|i| i.flatten().collect())?;

        Ok(docs)
    }

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError> {
        let url = Self::discover_batch_credential_endpoint(&self.message_client, &self.session_state.issuer_url)
            .await?
            .ok_or(IssuanceSessionError::NoBatchCredentialEndpoint)?;
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::DELETE).await?;

        self.message_client
            .reject(&url, &dpop_header, &access_token_header)
            .await?;

        Ok(())
    }

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview] {
        &self.session_state.normalized_credential_previews
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        &self.session_state.issuer_registration
    }
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    async fn request_credential(
        &self,
        credential_request: &CredentialRequest,
    ) -> Result<CredentialResponse, IssuanceSessionError> {
        let url = Self::discover_credential_endpoint(&self.message_client, &self.session_state.issuer_url).await?;
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::POST).await?;

        let response = self
            .message_client
            .request_credential(&url, credential_request, &dpop_header, &access_token_header)
            .await?;

        Ok(response)
    }

    async fn request_batch_credentials(
        &self,
        credential_requests: VecNonEmpty<CredentialRequest>,
        wte_disclosure: Option<WteDisclosure>,
        poa: Option<Poa>,
    ) -> Result<Vec<CredentialResponse>, IssuanceSessionError> {
        let url = Self::discover_batch_credential_endpoint(&self.message_client, &self.session_state.issuer_url)
            .await?
            .ok_or(IssuanceSessionError::NoBatchCredentialEndpoint)?;
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::POST).await?;

        let expected_response_count = credential_requests.len().get();
        let responses = self
            .message_client
            .request_credentials(
                &url,
                &CredentialRequests {
                    credential_requests,
                    attestations: wte_disclosure,
                    poa,
                },
                &dpop_header,
                &access_token_header,
            )
            .await?;

        // The server must have responded with enough credential responses, N, so that the caller has exactly enough
        // responses for all copies of all credentials constructed.
        if responses.credential_responses.len() != expected_response_count {
            return Err(IssuanceSessionError::UnexpectedCredentialResponseCount {
                found: responses.credential_responses.len(),
                expected: expected_response_count,
            });
        }

        Ok(responses.credential_responses)
    }
}

impl CredentialResponse {
    /// Create a credential out of the credential response. Also verifies the credential.
    fn into_credential<K: CredentialEcdsaKey>(
        self,
        key_id: String,
        verifying_key: &VerifyingKey,
        preview: &NormalizedCredentialPreview,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<IssuedCredential, IssuanceSessionError> {
        match self {
            CredentialResponse::MsoMdoc {
                credential: issuer_signed,
            } => {
                let CborBase64(issuer_signed) = *issuer_signed;

                // Calculate the minimum of all the lengths of the random bytes
                // included in the attributes of `IssuerSigned`. If this value
                // is too low, we should not accept the attributes.
                if let Some(name_spaces) = issuer_signed.name_spaces.as_ref() {
                    let min_random_length = name_spaces
                        .as_ref()
                        .values()
                        .flat_map(|attributes| attributes.as_ref().iter().map(|TaggedBytes(item)| item.random.len()))
                        .min();

                    if let Some(min_random_length) = min_random_length {
                        if min_random_length < ATTR_RANDOM_LENGTH {
                            return Err(IssuanceSessionError::AttributeRandomLength(
                                min_random_length,
                                ATTR_RANDOM_LENGTH,
                            ));
                        }
                    }
                }

                let credential_issuer_certificate = &issuer_signed
                    .issuer_auth
                    .signing_cert()
                    .map_err(IssuanceSessionError::IssuerCertificate)?;

                let credential_metadata_documents = &issuer_signed
                    .type_metadata_documents()
                    .map_err(IssuanceSessionError::Metadata)?;

                // Check that the collection of metadata received in the mdoc unsigned header
                // is the same as the one received for the preview. Note that the type metadata
                // integrity is checked for all the copies of a credential at once, so we do not
                // need to do that here.
                // TODO: remove this check in PVW-4320 when metadata has been removed from the mdoc
                if credential_metadata_documents != &preview.raw_metadata {
                    return Err(IssuanceSessionError::MetadataMismatch);
                }

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new::<K>(key_id, issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(IssuanceSessionError::MdocVerification)?;

                let issued_credential_payload = mdoc.clone().into_credential_payload(&preview.normalized_metadata)?;

                Self::validate_credential(
                    preview,
                    verifying_key,
                    issued_credential_payload,
                    credential_issuer_certificate,
                )?;

                Ok(IssuedCredential::MsoMdoc(Box::new(mdoc)))
            }
            CredentialResponse::SdJwt { credential } => {
                let issuer_pubkey = preview.content.issuer_certificate.public_key();

                let sd_jwt =
                    SdJwt::parse_and_verify(&credential, &EcdsaDecodingKey::from(issuer_pubkey), &Sha256Hasher)?;

                let credential_issuer_certificate = sd_jwt
                    .issuer_certificate()
                    .ok_or(IssuanceSessionError::MissingIssuerCertificate)?;

                let issued_credential_payload = sd_jwt.clone().into_credential_payload(&preview.normalized_metadata)?;

                Self::validate_credential(
                    preview,
                    verifying_key,
                    issued_credential_payload,
                    credential_issuer_certificate,
                )?;

                Ok(IssuedCredential::SdJwt(Box::new(sd_jwt)))
            }
        }
    }

    fn validate_credential(
        preview: &NormalizedCredentialPreview,
        holder_pubkey: &VerifyingKey,
        credential_payload: CredentialPayload,
        credential_issuer_certificate: &BorrowingCertificate,
    ) -> Result<(), IssuanceSessionError> {
        let NormalizedCredentialPreview { content, .. } = preview;

        let RequiredKeyBinding::Jwk(jwk) = credential_payload.confirmation_key;
        if jwk_to_p256(&jwk)? != *holder_pubkey {
            return Err(IssuanceSessionError::PublicKeyMismatch);
        }

        // The issuer certificate inside the mdoc has to equal the one that the issuer previously announced
        // in the credential preview.
        if credential_issuer_certificate != &content.issuer_certificate {
            return Err(IssuanceSessionError::IssuerMismatch);
        }

        // Check that our mdoc contains exactly the attributes the issuer said it would have.
        // Note that this also means that the mdoc's attributes must match the received metadata,
        // as both the metadata and attributes are the same as when we checked this for the preview.
        if credential_payload.previewable_payload != content.credential_payload {
            return Err(IssuanceSessionError::IssuedCredentialMismatch {
                actual: Box::new(credential_payload.previewable_payload),
                expected: Box::new(content.credential_payload.clone()),
            });
        }

        Ok(())
    }
}

impl IssuanceState {
    async fn auth_headers(&self, url: Url, method: Method) -> Result<(String, String), IssuanceSessionError> {
        let dpop_header = Dpop::new(
            &self.dpop_private_key,
            url,
            method,
            Some(&self.access_token),
            self.dpop_nonce.clone(),
        )
        .await?;

        let access_token_header = "DPoP ".to_string() + self.access_token.as_ref();

        Ok((dpop_header.into(), access_token_header))
    }
}

#[cfg(any(test, feature = "test"))]
pub async fn mock_wte<KF>(key_factory: &KF, privkey: &SigningKey) -> JwtCredential<WteClaims>
where
    KF: KeyFactory,
{
    use crypto::keys::EcdsaKey;
    use crypto::keys::WithIdentifier;
    use jwt::credential::JwtCredentialClaims;

    let wte_privkey = key_factory.generate_new().await.unwrap();

    let wte = JwtCredentialClaims::new_signed(
        &wte_privkey.verifying_key().await.unwrap(),
        privkey,
        "iss".to_string(),
        None,
        WteClaims::new(),
    )
    .await
    .unwrap();

    JwtCredential::new_unverified::<KF::Key>(wte_privkey.identifier().to_string(), wte)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;
    use std::sync::Arc;
    use std::vec;

    use assert_matches::assert_matches;
    use chrono::Utc;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_bytes::ByteBuf;
    use ssri::Integrity;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::auth::LocalizedStrings;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::qualification::AttestationQualification;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::KeyPair;
    use crypto::x509::CertificateError;
    use jwt::jwk;
    use mdoc::utils::serialization::CborBase64;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::IssuerSigned;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;

    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::token::CredentialPreview;
    use crate::token::TokenResponse;
    use crate::Format;

    use super::*;

    fn mock_openid_message_client() -> MockVcMessageClient {
        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client
            .expect_discover_metadata()
            .returning(|url| Ok(IssuerMetadata::new_mock(url)));
        mock_msg_client
            .expect_discover_oauth_metadata()
            .returning(|url| Ok(oidc::Config::new_mock(url)));
        mock_msg_client
            .expect_client_id()
            .return_const(MOCK_WALLET_CLIENT_ID.to_string());

        mock_msg_client
    }

    fn test_start_issuance(
        ca: &Ca,
        trust_anchor: TrustAnchor,
        credential_payloads: Vec<CredentialPayload>,
        type_metadata: TypeMetadata,
        formats: Vec<Format>,
    ) -> Result<HttpIssuanceSession<MockVcMessageClient>, IssuanceSessionError> {
        let issuance_key = generate_issuer_mock(ca, IssuerRegistration::new_mock().into()).unwrap();

        let mut mock_msg_client = mock_openid_message_client();
        mock_msg_client
            .expect_request_token()
            .return_once(move |_url, _token_request, _dpop_header| {
                let (_, _, type_metadata) = TypeMetadataDocuments::from_single_example(type_metadata);

                let previews = credential_payloads
                    .into_iter()
                    .map(|credential_payload| CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format: formats
                                .iter()
                                .map(|format| (*format, NonZeroU8::new(1).unwrap()))
                                .collect(),
                            credential_payload: credential_payload.previewable_payload,
                            issuer_certificate: issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    })
                    .collect_vec();

                let token_response = TokenResponseWithPreviews {
                    token_response: TokenResponse::new("access_token".to_string().into(), "c_nonce".to_string()),
                    credential_previews: VecNonEmpty::try_from(previews).unwrap(),
                };

                Ok((token_response, None))
            });

        HttpIssuanceSession::start_issuance(
            mock_msg_client,
            "https://example.com".parse().unwrap(),
            TokenRequest::new_mock(),
            &[trust_anchor],
        )
        .now_or_never()
        .unwrap()
    }

    #[test]
    fn test_start_issuance_ok() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let session = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            vec![CredentialPayload::example_family_name()],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect("starting issuance session should succeed");

        let NormalizedCredentialPreview {
            content,
            normalized_metadata,
            ..
        } = &session.normalized_credential_preview()[0];

        assert_matches!(
                &content.credential_payload.attributes["family_name"],
                Attribute::Single(AttributeValue::Text(v)) if v == "De Bruijn");

        assert_eq!(
            *normalized_metadata,
            TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example())
                .2
                .into_normalized(&content.credential_payload.attestation_type)
                .unwrap()
                .0
        );
    }

    #[test]
    fn test_start_issuance_untrusted_credential_preview() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            other_ca.to_trust_anchor(),
            vec![CredentialPayload::example_family_name()],
            TypeMetadata::pid_example(),
            vec![Format::MsoMdoc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            IssuanceSessionError::CredentialPreview(CredentialPreviewError::Certificate(
                CertificateError::Verification(_)
            ))
        );
    }

    #[test]
    fn test_start_issuance_type_metadata_verification_error() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            vec![CredentialPayload::example_empty(
                SigningKey::random(&mut OsRng).verifying_key(),
            )],
            TypeMetadata::empty_example_with_attestation_type("other_attestation_type"),
            vec![Format::MsoMdoc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, IssuanceSessionError::TypeMetadataVerification(_));
    }

    #[test]
    fn test_start_issuance_error_unsupported_credential_format() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            vec![CredentialPayload::example_empty(
                SigningKey::random(&mut OsRng).verifying_key(),
            )],
            TypeMetadata::pid_example(),
            vec![Format::AcVc, Format::MsoMdoc, Format::JwtVc],
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            IssuanceSessionError::UnsupportedCredentialFormat(attestation_type, formats)
                if attestation_type == "urn:eudi:pid:nl:1" && formats == HashSet::from([Format::JwtVc, Format::AcVc])
        );
    }

    #[test]
    fn test_start_issuance_error_different_issuer_registrations() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let issuance_key = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();
        let mut different_org = IssuerRegistration::new_mock();
        different_org.organization.display_name = LocalizedStrings::from(vec![("en", "different org name")]);
        let different_issuance_key = generate_issuer_mock(&ca, different_org.into()).unwrap();

        let payload = CredentialPayload::example_empty(SigningKey::random(&mut OsRng).verifying_key());
        let copies_per_format: IndexMap<Format, NonZeroU8> = IndexMap::from_iter([
            (Format::MsoMdoc, NonZeroU8::new(1).unwrap()),
            (Format::SdJwt, NonZeroU8::new(1).unwrap()),
        ]);

        let mut mock_msg_client = mock_openid_message_client();
        mock_msg_client
            .expect_request_token()
            .return_once(move |_url, _token_request, _dpop_header| {
                let (_, _, type_metadata) = TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example());

                let previews = vec![
                    CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format: copies_per_format.clone(),
                            credential_payload: payload.previewable_payload.clone(),
                            issuer_certificate: issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    },
                    CredentialPreview {
                        content: CredentialPreviewContent {
                            copies_per_format,
                            credential_payload: payload.previewable_payload,
                            issuer_certificate: different_issuance_key.certificate().clone(),
                        },
                        type_metadata: type_metadata.clone(),
                    },
                ];

                let token_response = TokenResponseWithPreviews {
                    token_response: TokenResponse::new("access_token".to_string().into(), "c_nonce".to_string()),
                    credential_previews: VecNonEmpty::try_from(previews).unwrap(),
                };

                Ok((token_response, None))
            });

        let error = HttpIssuanceSession::start_issuance(
            mock_msg_client,
            "https://example.com".parse().unwrap(),
            TokenRequest::new_mock(),
            &[ca.to_trust_anchor()],
        )
        .now_or_never()
        .unwrap()
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, IssuanceSessionError::DifferentIssuerRegistrations(_));
    }

    /// Return a new session ready for `accept_issuance()`.
    fn new_session_state(normalized_credential_previews: Vec<NormalizedCredentialPreview>) -> IssuanceState {
        let credential_request_types = credential_request_types_from_preview(&normalized_credential_previews).unwrap();

        IssuanceState {
            access_token: "access_token".to_string().into(),
            c_nonce: "c_nonce".to_string(),
            normalized_credential_previews,
            credential_request_types,
            issuer_registration: IssuerRegistration::new_mock(),
            issuer_url: "https://issuer.example.com".parse().unwrap(),
            dpop_private_key: SigningKey::random(&mut OsRng),
            dpop_nonce: Some("dpop_nonce".to_string()),
        }
    }

    #[derive(super::Debug, Clone)]
    struct MockCredentialSigner {
        pub trust_anchor: TrustAnchor<'static>,
        issuer_key: Arc<KeyPair>,
        metadata_documents: TypeMetadataDocuments,
        metadata_integrity: Integrity,
        previewable_payload: PreviewableCredentialPayload,
    }

    impl MockCredentialSigner {
        pub fn new_with_preview_state() -> (Self, NormalizedCredentialPreview) {
            let credential_payload = CredentialPayload::example_family_name();
            let type_metadata = TypeMetadata::example_with_claim_name(
                &credential_payload.previewable_payload.attestation_type,
                "family_name",
                JsonSchemaPropertyType::String,
                None,
            );

            Self::from_metadata_and_payload_with_preview_data(type_metadata, credential_payload)
        }

        pub fn from_metadata_and_payload_with_preview_data(
            type_metadata: TypeMetadata,
            credential_payload: CredentialPayload,
        ) -> (Self, NormalizedCredentialPreview) {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let trust_anchor = ca.to_trust_anchor().to_owned();

            let issuer_registration = IssuerRegistration::new_mock();
            let issuer_key = generate_issuer_mock(&ca, Some(issuer_registration.clone())).unwrap();
            let issuer_certificate = issuer_key.certificate().clone();

            let (attestation_type, metadata_integrity, metadata_documents) =
                TypeMetadataDocuments::from_single_example(type_metadata);
            let (normalized_metadata, raw_metadata) =
                metadata_documents.clone().into_normalized(&attestation_type).unwrap();

            let signer = Self {
                trust_anchor,
                issuer_key: Arc::new(issuer_key),
                metadata_documents,
                metadata_integrity,
                previewable_payload: credential_payload.previewable_payload.clone(),
            };

            let preview = NormalizedCredentialPreview {
                content: CredentialPreviewContent {
                    copies_per_format: IndexMap::from([(Format::MsoMdoc, NonZeroU8::new(1).unwrap())]),
                    credential_payload: credential_payload.previewable_payload,
                    issuer_certificate,
                },
                normalized_metadata,
                raw_metadata,
            };

            (signer, preview)
        }

        pub fn into_response_from_request(self, request: &CredentialRequest) -> CredentialResponse {
            let proof_jwt = match request.proof.as_ref().unwrap() {
                CredentialRequestProof::Jwt { jwt } => jwt,
            };
            let holder_public_key =
                jwk::jwk_to_p256(&jsonwebtoken::decode_header(&proof_jwt.0).unwrap().jwk.unwrap()).unwrap();

            self.into_response_from_holder_public_key(&holder_public_key)
        }

        pub fn into_response_from_holder_public_key(self, holder_public_key: &VerifyingKey) -> CredentialResponse {
            let (issuer_signed, _) = IssuerSigned::sign(
                self.previewable_payload.try_into().unwrap(),
                self.metadata_integrity,
                &self.metadata_documents,
                holder_public_key,
                &self.issuer_key,
            )
            .now_or_never()
            .unwrap()
            .unwrap();

            CredentialResponse::MsoMdoc {
                credential: Box::new(issuer_signed.into()),
            }
        }
    }

    /// Check consistency and validity of the input of the /(batch_)credential endpoints.
    fn check_credential_endpoint_input(
        url: &Url,
        session_state: &IssuanceState,
        dpop_header: &str,
        access_token_header: &str,
        attestations: &Option<WteDisclosure>,
        use_wte: bool,
    ) {
        assert_eq!(
            access_token_header,
            "DPoP ".to_string() + session_state.access_token.as_ref()
        );

        Dpop::from(dpop_header.to_string())
            .verify_expecting_key(
                session_state.dpop_private_key.verifying_key(),
                url,
                &Method::POST,
                Some(&session_state.access_token),
                session_state.dpop_nonce.as_deref(),
            )
            .unwrap();

        if use_wte != attestations.is_some() {
            panic!("unexpected WTE usage");
        }
    }

    #[rstest]
    fn test_accept_issuance(#[values(true, false)] use_wte: bool, #[values(true, false)] multiple_creds: bool) {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();
        let key_factory = MockRemoteKeyFactory::default();

        let wte = if use_wte {
            Some(
                mock_wte(&key_factory, &SigningKey::random(&mut OsRng))
                    .now_or_never()
                    .unwrap(),
            )
        } else {
            None
        };

        let session_state = new_session_state(if multiple_creds {
            vec![preview_data.clone(), preview_data]
        } else {
            vec![preview_data]
        });

        let mut mock_msg_client = mock_openid_message_client();

        // The client must use `request_credentials()` (which uses `/batch_credentials`) iff more than one credential
        // is being issued, and `request_credential()` instead (which uses `/credential`).
        if multiple_creds {
            mock_msg_client.expect_request_credentials().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_requests, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state,
                        dpop_header,
                        access_token_header,
                        &credential_requests.attestations,
                        use_wte,
                    );

                    let credential_responses = credential_requests
                        .credential_requests
                        .iter()
                        .zip(itertools::repeat_n(
                            signer,
                            credential_requests.credential_requests.len().get(),
                        ))
                        .map(|(request, signer)| signer.into_response_from_request(request))
                        .collect();

                    Ok(CredentialResponses { credential_responses })
                }
            });
        } else {
            mock_msg_client.expect_request_credential().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_request, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state,
                        dpop_header,
                        access_token_header,
                        &credential_request.attestations,
                        use_wte,
                    );

                    let response = signer.into_response_from_request(credential_request);

                    Ok(response)
                }
            });
        }

        // _ is an error because our mock does not behave like an actual issuer should, but it doesn't matter
        // because we are just inspecting what the client sent in this test with the expectation above.
        let _ = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&[trust_anchor], &key_factory, wte)
        .now_or_never();
    }

    #[test]
    fn test_accept_issuance_wrong_response_count() {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();

        let mut mock_msg_client = mock_openid_message_client();

        mock_msg_client.expect_request_credentials().return_once(
            |_url, credential_requests, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_requests.credential_requests.first());
                // Return one credential response.
                let responses = CredentialResponses {
                    credential_responses: vec![response],
                };

                Ok(responses)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(vec![preview_data.clone(), preview_data]),
        }
        .accept_issuance(&[trust_anchor], &MockRemoteKeyFactory::default(), None)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            IssuanceSessionError::UnexpectedCredentialResponseCount { found: 1, expected: 2 }
        );
    }

    #[test]
    fn test_accept_issuance_credential_payload_error() {
        let (signer, preview_data) = MockCredentialSigner::from_metadata_and_payload_with_preview_data(
            TypeMetadata::example_with_claim_name(
                "urn:eudi:pid:nl:1",
                "family_name",
                JsonSchemaPropertyType::String,
                None,
            ),
            CredentialPayload::example_with_attribute(
                "family_name",
                AttributeValue::Integer(1),
                SigningKey::random(&mut OsRng).verifying_key(),
            ),
        );
        let trust_anchor = signer.trust_anchor.clone();

        let session_state = new_session_state(vec![preview_data]);

        let mut mock_msg_client = mock_openid_message_client();

        mock_msg_client.expect_request_credential().times(1).return_once({
            move |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_request);

                Ok(response)
            }
        });

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&[trust_anchor], &MockRemoteKeyFactory::default(), None)
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, IssuanceSessionError::MdocCredentialPayload(_));
    }

    fn mock_credential_response() -> (
        CredentialResponse,
        NormalizedCredentialPreview,
        VerifyingKey,
        TrustAnchor<'static>,
    ) {
        let (signer, preview_data) = MockCredentialSigner::new_with_preview_state();
        let trust_anchor = signer.trust_anchor.clone();
        let holder_public_key = *SigningKey::random(&mut OsRng).verifying_key();
        let credential_response = signer.into_response_from_holder_public_key(&holder_public_key);

        (credential_response, preview_data, holder_public_key, trust_anchor)
    }

    #[test]
    fn test_credential_response_into_mdoc() {
        let (credential_response, preview_data, holder_public_key, trust_anchor) = mock_credential_response();

        let _issued_credential = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &[trust_anchor],
            )
            .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[test]
    fn test_credential_response_into_mdoc_public_key_mismatch_error() {
        let (credential_response, preview_data, _, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = *SigningKey::random(&mut OsRng).verifying_key();
        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &other_public_key,
                &preview_data,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::PublicKeyMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credential_response, preview_data, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` from a response
        // that contains insufficient random data should fail.
        let credential_response = match credential_response {
            CredentialResponse::MsoMdoc { mut credential } => {
                let CborBase64(ref mut credential_inner) = *credential;
                let name_spaces = credential_inner.name_spaces.as_mut().unwrap();

                name_spaces.modify_first_attributes(|attributes| {
                    let TaggedBytes(first_item) = attributes.first_mut().unwrap();

                    first_item.random = ByteBuf::from(b"12345");
                });

                CredentialResponse::MsoMdoc { credential }
            }
            CredentialResponse::SdJwt { .. } => panic!("unsupported credential request format"),
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH)
        );
    }

    #[test]
    fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error() {
        let (credential_response, normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_issuance_key = generate_issuer_mock(&other_ca, IssuerRegistration::new_mock().into()).unwrap();
        let preview_data = NormalizedCredentialPreview {
            content: CredentialPreviewContent {
                issuer_certificate: other_issuance_key.certificate().clone(),
                ..normalized_preview.content
            },
            ..normalized_preview
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuerMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_issuer_metadata_mismatch_error() {
        let (credential_response, normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` using different metadata
        // in the preview than is contained within the response should fail.
        let (attestation_type, _, different_metadata_documents) =
            TypeMetadataDocuments::from_single_example(TypeMetadata::empty_example_with_attestation_type("different"));
        let (different_normalized, different_raw) =
            different_metadata_documents.into_normalized(&attestation_type).unwrap();

        let preview_data = NormalizedCredentialPreview {
            normalized_metadata: different_normalized,
            raw_metadata: different_raw,
            ..normalized_preview
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::MetadataMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credential_response, normalized_preview, holder_public_key, _) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &holder_public_key, &normalized_preview, &[])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::MdocVerification(_));
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credential_response, mut normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail.
        let attributes = CredentialPayload::example_with_attributes(
            vec![
                ("new", AttributeValue::Bool(true)),
                ("family_name", AttributeValue::Text(String::from("De Bruijn"))),
            ],
            SigningKey::random(&mut OsRng).verifying_key(),
        )
        .previewable_payload
        .attributes;
        normalized_preview.content.credential_payload.attributes = attributes;

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_issuer_mismatch_error() {
        let (credential_response, mut normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` with a different `issuer_uri` in the preview than
        // contained within the response should fail.
        normalized_preview.content.credential_payload.issuer = "https://other-issuer.example.com".parse().unwrap();

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_doctype_mismatch_error() {
        let (credential_response, mut normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        normalized_preview.content.credential_payload.attestation_type = String::from("other.attestation_type");

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error() {
        let (credential_response, mut normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.

        normalized_preview.content.credential_payload.not_before =
            Some((Utc::now() + chrono::Duration::days(1)).into());

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attestation_qualification_mismatch_error() {
        let (credential_response, mut normalized_preview, holder_public_key, trust_anchor) = mock_credential_response();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        normalized_preview.content.credential_payload.attestation_qualification = AttestationQualification::PubEAA;

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>(
                "key_id".to_string(),
                &holder_public_key,
                &normalized_preview,
                &[trust_anchor],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuedCredentialMismatch { .. });
    }
}
