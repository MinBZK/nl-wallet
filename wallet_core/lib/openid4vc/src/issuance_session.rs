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
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crypto::factory::KeyFactory;
use crypto::keys::CredentialEcdsaKey;
use error_category::ErrorCategory;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredential;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteClaims;
use jwt::Jwt;
use mdoc::holder::IssuedDocumentMismatchError;
use mdoc::holder::Mdoc;
use mdoc::identifiers::AttributeIdentifier;
use mdoc::utils::cose::CoseError;
use mdoc::utils::serialization::CborBase64;
use mdoc::utils::serialization::CborError;
use mdoc::utils::serialization::TaggedBytes;
use mdoc::ATTR_RANDOM_LENGTH;
use poa::factory::PoaFactory;
use poa::Poa;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecAtLeastTwoUnique;
use utils::vec_at_least::VecNonEmpty;

use crate::credential::CredentialCopies;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::credential::MdocCopies;
use crate::credential::WteDisclosure;
use crate::credential_formats::CredentialFormats;
use crate::credential_payload::CredentialPayload;
use crate::credential_payload::CredentialPayloadError;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::dpop::DPOP_HEADER_NAME;
use crate::dpop::DPOP_NONCE_HEADER_NAME;
use crate::metadata::IssuerMetadata;
use crate::oidc;
use crate::token::AccessToken;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewError;
use crate::token::TokenRequest;
use crate::token::TokenResponseWithPreviews;
use crate::CredentialErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;

pub type CredentialPreviewsNormalizedMetadata =
    Vec<(CredentialFormats<CredentialPreview>, Vec<NormalizedTypeMetadata>)>;
pub type CredentialPreviewsMetadataDocuments =
    Vec<(CredentialFormats<CredentialPreview>, Vec<SortedTypeMetadataDocuments>)>;

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
    #[error("CBOR (de)serialization error: {0}")]
    Cbor(#[from] CborError),
    #[error("base64 decoding failed: {0}")]
    #[category(pd)]
    Base64Error(#[from] base64::DecodeError),
    #[error("mismatch between issued and expected attributes: {0:?}")]
    IssuedMdocMismatch(IssuedDocumentMismatchError<AttributeIdentifier>),
    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] mdoc::Error),
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
    #[error("failed to get mdoc public key: {0}")]
    PublicKeyFromMdoc(#[source] mdoc::Error),
    #[error("received {found} responses, expected {expected}")]
    #[category(critical)]
    UnexpectedCredentialResponseCount { found: usize, expected: usize },
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
    #[error("received zero credential copies")]
    #[category(critical)]
    NoCredentialCopies,
    #[error("error constructing PoA: {0}")]
    #[category(pd)]
    Poa(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("error converting to a CredentialPayload: {0}")]
    CredentialPayload(#[from] CredentialPayloadError),
}

#[derive(Clone, Debug)]
pub enum IssuedCredential {
    MsoMdoc(Box<Mdoc>),
}

impl TryFrom<IssuedCredential> for Mdoc {
    type Error = IssuanceSessionError;

    fn try_from(value: IssuedCredential) -> Result<Self, Self::Error> {
        match value {
            IssuedCredential::MsoMdoc(mdoc) => Ok(*mdoc),
        }
    }
}

#[derive(Clone, Debug)]
pub enum IssuedCredentialCopies {
    MsoMdoc(MdocCopies),
}

impl IssuedCredentialCopies {
    pub fn len(&self) -> usize {
        match self {
            IssuedCredentialCopies::MsoMdoc(mdocs) => mdocs.len(),
        }
    }

    // Required by clippy
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> TryFrom<&'a IssuedCredentialCopies> for &'a MdocCopies {
    type Error = IssuanceSessionError;

    fn try_from(value: &'a IssuedCredentialCopies) -> Result<Self, Self::Error> {
        match &value {
            IssuedCredentialCopies::MsoMdoc(mdocs) => Ok(mdocs),
        }
    }
}

impl TryFrom<IssuedCredentialCopies> for MdocCopies {
    type Error = IssuanceSessionError;

    fn try_from(value: IssuedCredentialCopies) -> Result<Self, Self::Error> {
        match value {
            IssuedCredentialCopies::MsoMdoc(mdocs) => Ok(mdocs),
        }
    }
}

impl<T> TryFrom<VecNonEmpty<IssuedCredential>> for CredentialCopies<T>
where
    T: TryFrom<IssuedCredential>,
{
    type Error = <T as TryFrom<IssuedCredential>>::Error;

    fn try_from(creds: VecNonEmpty<IssuedCredential>) -> Result<Self, Self::Error> {
        let copies = creds
            .into_inner()
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<T>, _>>()?
            .try_into()
            .unwrap(); // We always have at least one credential because our input was nonempty

        Ok(copies)
    }
}

impl TryFrom<Vec<IssuedCredential>> for IssuedCredentialCopies {
    type Error = IssuanceSessionError;

    fn try_from(creds: Vec<IssuedCredential>) -> Result<Self, Self::Error> {
        let copies = match creds.first().ok_or(IssuanceSessionError::NoCredentialCopies)? {
            // We can unwrap in these arms because we just checked that we have at least one credential
            IssuedCredential::MsoMdoc(_) => {
                IssuedCredentialCopies::MsoMdoc(VecNonEmpty::try_from(creds).unwrap().try_into()?)
            }
        };

        Ok(copies)
    }
}

pub trait IssuanceSession<H = HttpVcMessageClient> {
    async fn start_issuance(
        message_client: H,
        base_url: BaseUrl,
        token_request: TokenRequest,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<(Self, CredentialPreviewsNormalizedMetadata), IssuanceSessionError>
    where
        Self: Sized;

    async fn accept_issuance<K, KF>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &KF,
        wte: Option<JwtCredential<WteClaims>>,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>
    where
        K: CredentialEcdsaKey,
        K: Eq + Hash,
        KF: KeyFactory<Key = K>,
        KF: PoaFactory<Key = K>;

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError>;
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

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    c_nonce: String,
    credential_previews: CredentialPreviewsMetadataDocuments,
    issuer_url: BaseUrl,
    #[debug(skip)]
    dpop_private_key: SigningKey,
    dpop_nonce: Option<String>,
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
    ) -> Result<(Self, CredentialPreviewsNormalizedMetadata), IssuanceSessionError> {
        let token_endpoint = Self::discover_token_endpoint(&message_client, &base_url).await?;

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, token_endpoint.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = message_client
            .request_token(&token_endpoint, &token_request, &dpop_header)
            .await?;

        let (previews_metadata, previews_documents) = token_response
            .credential_previews
            .into_iter()
            .map(|formats| {
                // Verify the issuer certificate against the trust anchors.
                formats.verify(trust_anchors)?;

                let (metadata, documents) = formats
                    .as_ref()
                    .iter()
                    .map(|preview| match preview {
                        CredentialPreview::MsoMdoc {
                            unsigned_mdoc,
                            type_metadata,
                            ..
                        } => {
                            // For each preview of each format, decode the metadata chain, given the preview's `vct`.
                            let (metadata, sorted_documents) =
                                type_metadata.clone().into_normalized(&unsigned_mdoc.doc_type)?;

                            // Check the unsigned mdoc payload against the received
                            // metadata by converting it to a `CredentialPayload`.
                            let _credential_payload =
                                CredentialPayload::from_unsigned_mdoc(unsigned_mdoc.clone(), &metadata)?;

                            Ok::<_, IssuanceSessionError>((metadata, sorted_documents))
                        }
                    })
                    .process_results(|iter| iter.unzip())?;

                Ok::<_, IssuanceSessionError>(((formats.clone(), metadata), (formats, documents)))
            })
            .process_results(|iter| iter.unzip())?;

        let session_state = IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response
                .token_response
                .c_nonce
                .ok_or(IssuanceSessionError::MissingNonce)?,
            credential_previews: previews_documents,
            issuer_url: base_url,
            dpop_private_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };
        Ok((issuance_client, previews_metadata))
    }

    async fn accept_issuance<K, KF>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &KF,
        wte: Option<JwtCredential<WteClaims>>,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>
    where
        K: CredentialEcdsaKey,
        K: Eq + Hash,
        KF: KeyFactory<Key = K>,
        KF: PoaFactory<Key = K>,
    {
        // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
        // has no support (yet) for issuance of multiple copies of multiple attestations.
        // We implement this below by simply flattening the relevant nested iterators when communicating with the
        // issuer.

        let credential_previews = self
            .session_state
            .credential_previews
            .iter()
            .flat_map(|(formats, _)| formats.flatten_copies())
            .collect_vec();

        // Generate the PoPs to be sent to the issuer, and the private keys with which they were generated
        // (i.e., the private key of the future mdoc).
        // If N is the total amount of copies of credentials to be issued, then this returns N key/proof pairs.
        // Note that N > 0 because self.session_state.credential_previews which we mapped above is NonEmpty<_>.
        let keys_and_proofs = CredentialRequestProof::new_multiple(
            self.session_state.c_nonce.clone(),
            self.message_client.client_id().to_string(),
            self.session_state.issuer_url.clone(),
            credential_previews.len().try_into().unwrap(),
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
        let (pubkeys, credential_requests): (Vec<_>, Vec<_>) =
            try_join_all(keys_and_proofs.into_iter().zip(credential_previews).map(
                |((key, response), credential_preview)| async move {
                    let pubkey = key
                        .verifying_key()
                        .await
                        .map_err(|e| IssuanceSessionError::VerifyingKeyFromPrivateKey(e.into()))?;
                    let id = key.identifier().to_string();
                    let cred_request = CredentialRequest {
                        credential_type: credential_preview.credential_request_type().into(),
                        proof: Some(response),
                        attestations: None, // We set this field below if necessary
                        poa: None,          // We set this field below if necessary
                    };
                    Ok::<_, IssuanceSessionError>(((pubkey, id), cred_request))
                },
            ))
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
            .credential_previews
            .iter()
            .map(|(formats, documents)| {
                formats
                    .as_ref()
                    .iter()
                    .zip(documents)
                    .map(|(preview, sorted_documents)| {
                        let copy_count: usize = preview.copy_count().into();

                        // Consume the amount of copies from the front of `responses_and_keys`.
                        let cred_copies = responses_and_pubkeys
                            .drain(..copy_count)
                            .map(|(cred_response, (pubkey, key_id))| {
                                // Convert the response into a credential, verifying it against both the
                                // trust anchors and the credential preview we received in the preview.
                                cred_response.into_credential::<K>(key_id, &pubkey, preview, trust_anchors)
                            })
                            .collect::<Result<Vec<IssuedCredential>, _>>()?;

                        // Verify that each of the resulting mdocs contain exactly the same metadata integrity digest.
                        let integrity = cred_copies
                            .iter()
                            .map(|cred_copy| match cred_copy {
                                IssuedCredential::MsoMdoc(mdoc) => {
                                    mdoc.type_metadata_integrity().map_err(IssuanceSessionError::Metadata)
                                }
                            })
                            .process_results(|iter| {
                                iter.dedup()
                                    .exactly_one()
                                    .map_err(|_| IssuanceSessionError::MetadataIntegrityInconsistent)
                            })??;

                        // Check that the integrity hash received in the MSO matches
                        // that of encoded JSON of the first metadata document.
                        sorted_documents.verify(integrity.clone())?;

                        cred_copies.try_into()
                    })
                    .collect::<Result<Vec<IssuedCredentialCopies>, _>>()
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
        preview: &CredentialPreview,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<IssuedCredential, IssuanceSessionError> {
        match self {
            CredentialResponse::MsoMdoc {
                credential: issuer_signed,
            } => {
                let CborBase64(issuer_signed) = *issuer_signed;
                let CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                } = preview;

                if issuer_signed
                    .public_key()
                    .map_err(IssuanceSessionError::PublicKeyFromMdoc)?
                    != *verifying_key
                {
                    return Err(IssuanceSessionError::PublicKeyMismatch);
                }

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

                // The issuer certificate inside the mdoc has to equal the one that the issuer previously announced
                // in the credential preview.
                if issuer_signed
                    .issuer_auth
                    .signing_cert()
                    .map_err(IssuanceSessionError::IssuerCertificate)?
                    != *issuer_certificate
                {
                    return Err(IssuanceSessionError::IssuerMismatch);
                }

                // Check that the metadata received in the mdoc unsigned header
                // is the same as the one received for the preview.
                let metadata_documents = issuer_signed
                    .type_metadata_documents()
                    .map_err(IssuanceSessionError::Metadata)?;
                if metadata_documents != *type_metadata {
                    return Err(IssuanceSessionError::MetadataMismatch);
                }

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new::<K>(key_id, issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(IssuanceSessionError::MdocVerification)?;

                // Check that our mdoc contains exactly the attributes the issuer said it would have.
                // Note that this also means that the mdoc's attributes must match the received metadata,
                // as both the metadata and attributes are the same as when we checked this for the preview.
                mdoc.compare_unsigned(unsigned_mdoc)
                    .map_err(IssuanceSessionError::IssuedMdocMismatch)?;

                Ok(IssuedCredential::MsoMdoc(Box::new(mdoc)))
            }
        }
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
    use assert_matches::assert_matches;
    use chrono::Utc;
    use futures::FutureExt;
    use rstest::rstest;
    use serde_bytes::ByteBuf;

    use crypto::factory::KeyFactory;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateError;
    use mdoc::holder::IssuedDocumentMismatchError;
    use mdoc::server_keys::generate::mock::generate_issuer_mock;
    use mdoc::test::data;
    use mdoc::unsigned::UnsignedMdoc;
    use mdoc::utils::issuer_auth::IssuerRegistration;
    use mdoc::utils::serialization::CborBase64;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::AttestationQualification;
    use mdoc::IssuerSigned;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;

    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::token::TokenResponse;

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

    async fn create_credential_response() -> (
        CredentialResponse,
        CredentialPreview,
        SortedTypeMetadataDocuments,
        TrustAnchor<'static>,
        VerifyingKey,
        MockRemoteKeyFactory,
    ) {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_key = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();
        let key_factory = MockRemoteKeyFactory::default();
        let trust_anchor = ca.to_trust_anchor().to_owned();

        let unsigned_mdoc = UnsignedMdoc::from(data::pid_family_name().into_first().unwrap());

        let metadata = TypeMetadata::example_with_claim_name(
            &unsigned_mdoc.doc_type,
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        );
        let (_, metadata_integrity, metadata_documents) = TypeMetadataDocuments::from_single_example(metadata);

        let mdoc_key = key_factory.generate_new().await.unwrap();
        let mdoc_public_key = mdoc_key.verifying_key();
        let issuer_signed = IssuerSigned::sign(
            unsigned_mdoc.clone(),
            metadata_integrity,
            &metadata_documents,
            mdoc_public_key.try_into().unwrap(),
            &issuance_key,
        )
        .await
        .unwrap();

        let credential_response = CredentialResponse::MsoMdoc {
            credential: Box::new(issuer_signed.into()),
        };

        let (_, sorted_documents) = metadata_documents
            .clone()
            .into_normalized(&unsigned_mdoc.doc_type)
            .unwrap();

        let preview = CredentialPreview::MsoMdoc {
            unsigned_mdoc,
            issuer_certificate: issuance_key.certificate().clone(),
            type_metadata: metadata_documents,
        };

        (
            credential_response,
            preview,
            sorted_documents,
            trust_anchor,
            *mdoc_public_key,
            key_factory,
        )
    }

    fn test_start_issuance(
        ca: &Ca,
        trust_anchor: TrustAnchor,
        type_metadata: TypeMetadata,
    ) -> Result<
        (
            HttpIssuanceSession<MockVcMessageClient>,
            CredentialPreviewsNormalizedMetadata,
        ),
        IssuanceSessionError,
    > {
        let issuance_key = generate_issuer_mock(ca, IssuerRegistration::new_mock().into()).unwrap();

        let mut mock_msg_client = mock_openid_message_client();
        mock_msg_client
            .expect_request_token()
            .return_once(move |_url, _token_request, _dpop_header| {
                let unsigned_mdoc = UnsignedMdoc::from(data::pid_family_name().into_first().unwrap());
                let (_, _, type_metadata) = TypeMetadataDocuments::from_single_example(type_metadata);

                let preview = CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate: issuance_key.certificate().clone(),
                    type_metadata,
                };

                let token_response = TokenResponseWithPreviews {
                    token_response: TokenResponse::new("access_token".to_string().into(), "c_nonce".to_string()),
                    credential_previews: VecNonEmpty::try_from(vec![CredentialFormats::try_new(
                        VecNonEmpty::try_from(vec![preview]).unwrap(),
                    )
                    .unwrap()])
                    .unwrap(),
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

        let (_, previews_with_metadata) = test_start_issuance(&ca, ca.to_trust_anchor(), TypeMetadata::pid_example())
            .expect("starting issuance session should succeed");

        let (formats, metadata) = &previews_with_metadata[0];
        match &formats.as_ref().as_ref()[0] {
            CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                type_metadata,
                ..
            } => {
                let first_attribute = &unsigned_mdoc.attributes.as_ref()[0][0];

                assert_eq!(first_attribute.name, "family_name");
                assert_eq!(first_attribute.value.as_text().unwrap(), "De Bruijn");

                assert_eq!(
                    *metadata.first().unwrap(),
                    type_metadata
                        .clone()
                        .into_normalized(&unsigned_mdoc.doc_type)
                        .unwrap()
                        .0
                );
            }
        }
    }

    #[test]
    fn test_start_issuance_untrusted_credential_preview() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(&ca, other_ca.to_trust_anchor(), TypeMetadata::pid_example())
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
            TypeMetadata::empty_example_with_attestation_type("other_attestation_type"),
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, IssuanceSessionError::TypeMetadataVerification(_));
    }

    #[test]
    fn test_start_issuance_type_credential_payload_error() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            ca.to_trust_anchor(),
            TypeMetadata::empty_example_with_attestation_type(data::PID),
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, IssuanceSessionError::CredentialPayload(_));
    }

    /// Return a new session ready for `accept_issuance()`.
    fn new_session_state(credential_previews: CredentialPreviewsMetadataDocuments) -> IssuanceState {
        IssuanceState {
            access_token: "access_token".to_string().into(),
            c_nonce: "c_nonce".to_string(),
            credential_previews,
            issuer_url: "https://issuer.example.com".parse().unwrap(),
            dpop_private_key: SigningKey::random(&mut OsRng),
            dpop_nonce: Some("dpop_nonce".to_string()),
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
    #[tokio::test]
    async fn test_accept_issuance(#[values(true, false)] use_wte: bool, #[values(true, false)] multiple_creds: bool) {
        let (cred_response, preview, sorted_documents, trust_anchor, _, key_factory) =
            create_credential_response().await;
        let wte = if use_wte {
            Some(mock_wte(&key_factory, &SigningKey::random(&mut OsRng)).await)
        } else {
            None
        };
        let format = CredentialFormats::try_new(VecNonEmpty::try_from(vec![preview]).unwrap()).unwrap();
        let format_with_documents = (format, vec![sorted_documents]);
        let session_state = new_session_state(if multiple_creds {
            vec![format_with_documents.clone(), format_with_documents]
        } else {
            vec![format_with_documents]
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
                    Ok(CredentialResponses {
                        credential_responses: vec![cred_response.clone(), cred_response],
                    })
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
                    Ok(cred_response)
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
        .await;
    }

    #[tokio::test]
    async fn test_accept_issuance_wrong_response_count() {
        let mut mock_msg_client = mock_openid_message_client();
        let (cred_response, preview, sorted_documents, trust_anchor, _, _) = create_credential_response().await;

        mock_msg_client.expect_request_credentials().return_once(
            |_url, _credential_requests, _dpop_header, _access_token_header| {
                Ok(CredentialResponses {
                    credential_responses: vec![cred_response], // return one credential response
                })
            },
        );

        let format = CredentialFormats::try_new(VecNonEmpty::try_from(vec![preview]).unwrap()).unwrap();
        let format_with_documents = (format, vec![sorted_documents]);
        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(vec![format_with_documents.clone(), format_with_documents]),
        }
        .accept_issuance(&[trust_anchor], &MockRemoteKeyFactory::default(), None)
        .await
        .unwrap_err();

        assert_matches!(
            error,
            IssuanceSessionError::UnexpectedCredentialResponseCount { found: 1, expected: 2 }
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        let _ = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_public_key_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, _, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = *SigningKey::random(&mut OsRng).verifying_key();
        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &other_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::PublicKeyMismatch);
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

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
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH)
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_issuance_key = generate_issuer_mock(&other_ca, IssuerRegistration::new_mock().into()).unwrap();
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate: _,
                type_metadata,
            } => CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate: other_issuance_key.certificate().clone(),
                type_metadata,
            },
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuerMismatch);
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issuer_metadata_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` using different metadata
        // in the preview than is contained within the response should fail.
        let (_, _, different_metadata_documents) =
            TypeMetadataDocuments::from_single_example(TypeMetadata::empty_example_with_attestation_type("different"));
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate,
                type_metadata: _,
            } => CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer_certificate,
                type_metadata: different_metadata_documents,
            },
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::MetadataMismatch);
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credential_response, preview, _, _, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::MdocVerification(_));
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                mut unsigned_mdoc,
                issuer_certificate,
                type_metadata,
            } => {
                let attributes = UnsignedMdoc::from(data::pid_full_name().into_first().unwrap()).attributes;
                unsigned_mdoc.attributes = attributes;
                CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                }
            }
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocMismatch(IssuedDocumentMismatchError::IssuedAttributesMismatch(missing, unexpected))
                if missing.len() == 1 && unexpected.is_empty()
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_issuer_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with a different `issuer_uri` in the preview than
        // contained within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                mut unsigned_mdoc,
                issuer_certificate,
                type_metadata,
            } => {
                unsigned_mdoc.issuer_uri = "https://other-issuer.example.com".parse().unwrap();
                CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                }
            }
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocMismatch(IssuedDocumentMismatchError::IssuedIssuerMismatch(_, _))
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_doctype_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                mut unsigned_mdoc,
                issuer_certificate,
                type_metadata,
            } => {
                unsigned_mdoc.doc_type = String::from("other.doc_type");
                CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                }
            }
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocMismatch(IssuedDocumentMismatchError::IssuedDoctypeMismatch(_, _))
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                mut unsigned_mdoc,
                issuer_certificate,
                type_metadata,
            } => {
                unsigned_mdoc.valid_from = (Utc::now() + chrono::Duration::days(1)).into();
                CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                }
            }
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocMismatch(IssuedDocumentMismatchError::IssuedValidityInfoMismatch(_, _))
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_attestation_qualification_mismatch_error() {
        let (credential_response, preview, _, trust_anchor, mdoc_public_key, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                mut unsigned_mdoc,
                issuer_certificate,
                type_metadata,
            } => {
                unsigned_mdoc.attestation_qualification = AttestationQualification::PubEAA;
                CredentialPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer_certificate,
                    type_metadata,
                }
            }
        };

        let error = credential_response
            .into_credential::<MockRemoteEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[trust_anchor])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocMismatch(
                IssuedDocumentMismatchError::IssuedAttestationQualificationMismatch(
                    AttestationQualification::PubEAA,
                    AttestationQualification::EAA,
                )
            )
        );
    }
}
