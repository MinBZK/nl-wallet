use std::{collections::VecDeque, fmt::Debug};

use futures::{future::try_join_all, TryFutureExt};
use itertools::Itertools;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use reqwest::{
    header::{ToStrError, AUTHORIZATION},
    Method,
};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use error_category::ErrorCategory;
use nl_wallet_mdoc::{
    holder::{IssuedAttributesMismatch, Mdoc, TrustAnchor},
    utils::{
        cose::CoseError,
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::{CborBase64, CborError, TaggedBytes},
        x509::CertificateError,
    },
    ATTR_RANDOM_LENGTH,
};
use wallet_common::{generator::TimeGenerator, jwt::JwtError, nonempty::NonEmpty, urls::BaseUrl};

use crate::{
    credential::{
        CredentialCopies, CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponse,
        CredentialResponses, MdocCopies,
    },
    dpop::{Dpop, DpopError, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    jwt::{JwkConversionError, JwtCredential, JwtCredentialError},
    metadata::IssuerMetadata,
    oidc,
    token::{AccessToken, CredentialPreview, TokenRequest, TokenResponseWithPreviews},
    CredentialErrorCode, ErrorResponse, Format, TokenErrorCode, NL_WALLET_CLIENT_ID,
};

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
    #[error("mismatch between issued and expected attributes in mdoc: {0}")]
    IssuedMdocAttributesMismatch(#[source] IssuedAttributesMismatch),
    #[error("mismatch between issued and expected attributes in JWT: {0}")]
    IssuedJwtAttributesMismatch(#[source] IssuedAttributesMismatch<String>),
    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] nl_wallet_mdoc::Error),
    #[error("jwt credential verification failed: {0}")]
    JwtCredentialVerification(#[from] JwtCredentialError),
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
    PublicKeyFromMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("received {found} responses, expected {expected}")]
    #[category(critical)]
    UnexpectedCredentialResponseCount { found: usize, expected: usize },
    #[error("error reading HTTP error: {0}")]
    #[category(pd)]
    HeaderToStr(#[from] ToStrError),
    #[error("error verifying certificate of credential preview: {0}")]
    Certificate(#[from] CertificateError),
    #[error("issuer contained in credential not equal to expected value")]
    #[category(critical)]
    IssuerMismatch,
    #[error("error retrieving issuer certificate from issued mdoc: {0}")]
    Cose(#[from] CoseError),
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
    #[error("unexpected credential format: expected {expected:?}, found {found:?}")]
    #[category(critical)]
    UnexpectedCredentialFormat { expected: Format, found: Format },
    #[error("received zero credential copies")]
    #[category(critical)]
    NoCredentialCopies,
}

#[derive(Clone, Debug)]
pub enum IssuedCredential {
    MsoMdoc(Mdoc),
    Jwt(JwtCredential),
}

impl From<&IssuedCredential> for Format {
    fn from(value: &IssuedCredential) -> Self {
        match value {
            IssuedCredential::MsoMdoc(_) => Format::MsoMdoc,
            IssuedCredential::Jwt(_) => Format::Jwt,
        }
    }
}

#[derive(Clone, Debug)]
pub enum IssuedCredentialCopies {
    MsoMdoc(MdocCopies),
    Jwt(CredentialCopies<JwtCredential>),
}

impl IssuedCredentialCopies {
    pub fn len(&self) -> usize {
        match self {
            IssuedCredentialCopies::MsoMdoc(mdocs) => mdocs.as_ref().len(),
            IssuedCredentialCopies::Jwt(jwts) => jwts.as_ref().len(),
        }
    }

    // Required by clippy
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<&IssuedCredentialCopies> for Format {
    fn from(value: &IssuedCredentialCopies) -> Self {
        match value {
            IssuedCredentialCopies::MsoMdoc(_) => Format::MsoMdoc,
            IssuedCredentialCopies::Jwt(_) => Format::Jwt,
        }
    }
}

impl<'a> TryFrom<&'a IssuedCredentialCopies> for &'a MdocCopies {
    type Error = IssuanceSessionError;

    fn try_from(value: &'a IssuedCredentialCopies) -> Result<Self, Self::Error> {
        match &value {
            IssuedCredentialCopies::MsoMdoc(mdocs) => Ok(mdocs),
            _ => Err(IssuanceSessionError::UnexpectedCredentialFormat {
                expected: Format::MsoMdoc,
                found: value.into(),
            }),
        }
    }
}

impl TryFrom<IssuedCredentialCopies> for MdocCopies {
    type Error = IssuanceSessionError;

    fn try_from(value: IssuedCredentialCopies) -> Result<Self, Self::Error> {
        match value {
            IssuedCredentialCopies::MsoMdoc(mdocs) => Ok(mdocs),
            _ => Err(IssuanceSessionError::UnexpectedCredentialFormat {
                expected: Format::MsoMdoc,
                found: (&value).into(),
            }),
        }
    }
}

impl TryFrom<Vec<IssuedCredential>> for IssuedCredentialCopies {
    type Error = IssuanceSessionError;

    fn try_from(creds: Vec<IssuedCredential>) -> Result<Self, Self::Error> {
        let copies = match creds.first().ok_or(IssuanceSessionError::NoCredentialCopies)? {
            IssuedCredential::MsoMdoc(_) => {
                let mdoc_copies = creds
                    .into_iter()
                    .map(|cred| match cred {
                        IssuedCredential::MsoMdoc(mdoc) => Ok(mdoc),
                        _ => Err(IssuanceSessionError::UnexpectedCredentialFormat {
                            expected: Format::MsoMdoc,
                            found: (&cred).into(),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap(); // we checked above that we always have at least one credential
                IssuedCredentialCopies::MsoMdoc(mdoc_copies)
            }
            IssuedCredential::Jwt(_) => {
                let jwt_copies = creds
                    .into_iter()
                    .map(|cred| match cred {
                        IssuedCredential::Jwt(jwt) => Ok(jwt),
                        _ => Err(IssuanceSessionError::UnexpectedCredentialFormat {
                            expected: Format::Jwt,
                            found: (&cred).into(),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap(); // we checked above that we always have at least one credential
                IssuedCredentialCopies::Jwt(jwt_copies)
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
    ) -> Result<(Self, Vec<CredentialPreview>), IssuanceSessionError>
    where
        Self: Sized;

    async fn accept_issuance<K: MdocEcdsaKey>(
        &self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: BaseUrl,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>;

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
    http_client: reqwest::Client,
}

impl From<reqwest::Client> for HttpVcMessageClient {
    fn from(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }
}

impl VcMessageClient for HttpVcMessageClient {
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

struct IssuanceState {
    access_token: AccessToken,
    c_nonce: String,
    credential_previews: NonEmpty<Vec<CredentialPreview>>,
    issuer_url: BaseUrl,
    dpop_private_key: SigningKey,
    dpop_nonce: Option<String>,
}

impl Debug for IssuanceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IssuanceState")
            .field("access_token", &self.access_token)
            .field("c_nonce", &self.c_nonce)
            .field("credential_previews", &self.credential_previews)
            .field("issuer_url", &self.issuer_url)
            .field("dpop_nonce", &self.dpop_nonce)
            .finish_non_exhaustive() // don't show dpop_private_key
    }
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
    ) -> Result<(Self, Vec<CredentialPreview>), IssuanceSessionError> {
        let token_endpoint = Self::discover_token_endpoint(&message_client, &base_url).await?;

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, token_endpoint.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = message_client
            .request_token(&token_endpoint, &token_request, &dpop_header)
            .await?;

        token_response
            .credential_previews
            .as_ref()
            .iter()
            .try_for_each(|preview| preview.verify(trust_anchors))?;

        let credential_previews = token_response.credential_previews.clone().into_inner();

        let session_state = IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response
                .token_response
                .c_nonce
                .ok_or(IssuanceSessionError::MissingNonce)?,
            credential_previews: token_response.credential_previews,
            issuer_url: base_url,
            dpop_private_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };
        Ok((issuance_client, credential_previews))
    }

    async fn accept_issuance<K: MdocEcdsaKey>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: BaseUrl,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError> {
        // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
        // has no support (yet) for issuance of multiple copies of multiple attestations.
        // We implement this below by simply flattening the relevant nested iterators when communicating with the
        // issuer.

        let types = self
            .session_state
            .credential_previews
            .as_ref()
            .iter()
            .flat_map(|preview| itertools::repeat_n(preview.into(), preview.copy_count().into()))
            .collect_vec();

        // Generate the PoPs to be sent to the issuer, and the private keys with which they were generated
        // (i.e., the private key of the future mdoc).
        // If N is the total amount of copies of credentials to be issued, then this returns N key/proof pairs.
        // Note that N > 0 because self.session_state.credential_previews which we mapped above is NonEmpty<_>.
        let keys_and_proofs = CredentialRequestProof::new_multiple(
            self.session_state.c_nonce.clone(),
            NL_WALLET_CLIENT_ID.to_string(),
            credential_issuer_identifier,
            types.len().try_into().unwrap(),
            key_factory,
        )
        .await?;

        // Split into N keys and N credential requests, so we can send the credential request proofs separately
        // to the issuer.
        let (pubkeys, credential_requests): (Vec<_>, Vec<_>) = try_join_all(
            keys_and_proofs
                .into_iter()
                .zip(types)
                .map(|((key, response), credential_type)| async move {
                    let pubkey = key
                        .verifying_key()
                        .await
                        .map_err(|e| IssuanceSessionError::VerifyingKeyFromPrivateKey(e.into()))?;
                    let id = key.identifier().to_string();
                    let cred_request = CredentialRequest {
                        credential_type,
                        proof: Some(response),
                    };
                    Ok::<_, IssuanceSessionError>(((pubkey, id), cred_request))
                }),
        )
        .await?
        .into_iter()
        .unzip();

        // Unwrapping is safe because N > 0, see above.
        let credential_requests = NonEmpty::new(credential_requests).unwrap();
        let responses = match credential_requests.as_ref().len() {
            1 => vec![self.request_credential(credential_requests.first()).await?],
            _ => self.request_batch_credentials(credential_requests).await?,
        };
        let mut responses_and_pubkeys: VecDeque<_> = responses.into_iter().zip(pubkeys).collect();

        let mdocs = self
            .session_state
            .credential_previews
            .as_ref()
            .iter()
            .map(|preview| {
                let copy_count: usize = preview.copy_count().into();

                // Consume the amount of copies from the front of `responses_and_keys`.
                let cred_copies = responses_and_pubkeys
                    .drain(..copy_count)
                    .map(|(cred_response, (pubkey, key_id))| {
                        // Convert the response into an credential, verifying it against both the
                        // trust anchors and the credential preview we received in the preview.
                        cred_response.into_credential::<K>(key_id, &pubkey, preview, trust_anchors)
                    })
                    .collect::<Result<Vec<IssuedCredential>, _>>()?;

                // For each preview we have an `IssuedCredentialCopies` instance.
                cred_copies.try_into()
            })
            .collect::<Result<_, IssuanceSessionError>>()?;

        Ok(mdocs)
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
        credential_requests: NonEmpty<Vec<CredentialRequest>>,
    ) -> Result<Vec<CredentialResponse>, IssuanceSessionError> {
        let url = Self::discover_batch_credential_endpoint(&self.message_client, &self.session_state.issuer_url)
            .await?
            .ok_or(IssuanceSessionError::NoBatchCredentialEndpoint)?;
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::POST).await?;

        let expected_response_count = credential_requests.as_ref().len();
        let responses = self
            .message_client
            .request_credentials(
                &url,
                &CredentialRequests { credential_requests },
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
    fn into_credential<K: MdocEcdsaKey>(
        self,
        key_id: String,
        verifying_key: &VerifyingKey,
        preview: &CredentialPreview,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<IssuedCredential, IssuanceSessionError> {
        match self {
            CredentialResponse::MsoMdoc {
                credential: CborBase64(issuer_signed),
            } => {
                let CredentialPreview::MsoMdoc { unsigned_mdoc, issuer } = preview else {
                    return Err(IssuanceSessionError::UnexpectedCredentialFormat {
                        expected: Format::MsoMdoc,
                        found: preview.into(),
                    });
                };

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
                if issuer_signed.issuer_auth.signing_cert()? != *issuer {
                    return Err(IssuanceSessionError::IssuerMismatch);
                }

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new::<K>(key_id, issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(IssuanceSessionError::MdocVerification)?;

                // Check that our mdoc contains exactly the attributes the issuer said it would have
                mdoc.compare_unsigned(unsigned_mdoc)
                    .map_err(IssuanceSessionError::IssuedMdocAttributesMismatch)?;

                Ok(IssuedCredential::MsoMdoc(mdoc))
            }
            CredentialResponse::Jwt { credential } => {
                let (cred, cred_claims) = JwtCredential::new::<K>(key_id, credential, trust_anchors)?;

                let CredentialPreview::Jwt {
                    claims: expected_claims,
                    ..
                } = preview
                else {
                    return Err(IssuanceSessionError::UnexpectedCredentialFormat {
                        expected: Format::Jwt,
                        found: preview.into(),
                    });
                };

                if cred_claims.contents.iss != expected_claims.iss {
                    return Err(IssuanceSessionError::IssuerMismatch);
                }

                cred_claims
                    .contents
                    .compare_attributes(expected_claims)
                    .map_err(IssuanceSessionError::IssuedJwtAttributesMismatch)?;

                Ok(IssuedCredential::Jwt(cred))
            }
        }
    }
}

impl IssuanceState {
    async fn auth_headers(&self, url: Url, method: reqwest::Method) -> Result<(String, String), IssuanceSessionError> {
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

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_bytes::ByteBuf;

    use nl_wallet_mdoc::{
        server_keys::KeyPair,
        software_key_factory::SoftwareKeyFactory,
        test::data,
        unsigned::UnsignedMdoc,
        utils::{
            issuer_auth::IssuerRegistration,
            serialization::{CborBase64, TaggedBytes},
            x509::Certificate,
        },
        IssuerSigned,
    };
    use wallet_common::{
        keys::{software::SoftwareEcdsaKey, EcdsaKey},
        nonempty::NonEmpty,
    };

    use crate::token::TokenResponse;

    use super::*;

    fn mock_openid_message_client() -> MockVcMessageClient {
        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client
            .expect_discover_metadata()
            .returning(|url| Ok(IssuerMetadata::new_mock(url.clone())));
        mock_msg_client
            .expect_discover_oauth_metadata()
            .returning(|url| Ok(oidc::Config::new_mock(url)));
        mock_msg_client
    }

    async fn create_credential_response() -> (CredentialResponse, CredentialPreview, Certificate, VerifyingKey) {
        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();
        let key_factory = SoftwareKeyFactory::default();

        let unsigned_mdoc = UnsignedMdoc::from(data::pid_family_name().into_first().unwrap());
        let preview = CredentialPreview::MsoMdoc {
            unsigned_mdoc: unsigned_mdoc.clone(),
            issuer: issuance_key.certificate().clone(),
        };

        let mdoc_key = key_factory.generate_new().await.unwrap();
        let mdoc_public_key = mdoc_key.verifying_key().await.unwrap();
        let issuer_signed = IssuerSigned::sign(unsigned_mdoc, (&mdoc_public_key).try_into().unwrap(), &issuance_key)
            .await
            .unwrap();
        let credential_response = CredentialResponse::MsoMdoc {
            credential: issuer_signed.into(),
        };

        (credential_response, preview, ca.certificate().clone(), mdoc_public_key)
    }

    #[tokio::test]
    async fn test_start_issuance_untrusted_credential_preview() {
        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let ca_cert = ca.certificate();
        let trust_anchors = &[(ca_cert.try_into().unwrap())];

        let mut mock_msg_client = mock_openid_message_client();
        mock_msg_client
            .expect_request_token()
            .return_once(|_url, _token_request, _dpop_header| {
                // Generate the credential previews with some other CA than what the
                // HttpIssuanceSession::start_issuance() will accept
                let ca = KeyPair::generate_issuer_mock_ca().unwrap();
                let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

                let preview = CredentialPreview::MsoMdoc {
                    unsigned_mdoc: UnsignedMdoc::from(data::pid_family_name().into_first().unwrap()),
                    issuer: issuance_key.certificate().clone(),
                };

                Ok((
                    TokenResponseWithPreviews {
                        token_response: TokenResponse::new("access_token".to_string().into(), "c_nonce".to_string()),
                        credential_previews: NonEmpty::new(vec![preview]).unwrap(),
                    },
                    None,
                ))
            });

        let token_request = TokenRequest::new_mock();

        let error = HttpIssuanceSession::start_issuance(
            mock_msg_client,
            "https://example.com".parse().unwrap(),
            token_request,
            trust_anchors,
        )
        .await
        .unwrap_err();

        assert_matches!(
            error,
            IssuanceSessionError::Certificate(CertificateError::Verification(_))
        )
    }

    #[tokio::test]
    async fn test_accept_issuance_wrong_response_count() {
        let (cred_response, preview, ca_cert, _) = create_credential_response().await;
        let trust_anchors = &[((&ca_cert).try_into().unwrap())];

        let mut mock_msg_client = mock_openid_message_client();
        mock_msg_client
            .expect_request_token()
            .return_once(|_url, _token_request, _dpop_header| {
                Ok((
                    TokenResponseWithPreviews {
                        token_response: TokenResponse::new("access_token".to_string().into(), "c_nonce".to_string()),
                        // return two previews
                        credential_previews: NonEmpty::new(vec![preview.clone(), preview]).unwrap(),
                    },
                    Some("dpop_nonce".to_string()),
                ))
            });
        mock_msg_client.expect_request_credentials().return_once(
            |_url, _credential_requests, _dpop_header, _access_token_header| {
                Ok(CredentialResponses {
                    credential_responses: vec![cred_response], // return one credential response
                })
            },
        );

        let token_request = TokenRequest::new_mock();

        let (client, previews) = HttpIssuanceSession::start_issuance(
            mock_msg_client,
            "https://example.com".parse().unwrap(),
            token_request,
            trust_anchors,
        )
        .await
        .unwrap();

        assert_eq!(previews.len(), 2);

        let error = client
            .accept_issuance(
                trust_anchors,
                SoftwareKeyFactory::default(),
                "https://example.com".parse().unwrap(),
            )
            .await
            .unwrap_err();

        assert_matches!(
            error,
            IssuanceSessionError::UnexpectedCredentialResponseCount { found: 1, expected: 2 }
        );
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc() {
        let (credential_response, preview, ca_cert, mdoc_public_key) = create_credential_response().await;

        let _ = credential_response
            .into_credential::<SoftwareEcdsaKey>(
                "key_id".to_string(),
                &mdoc_public_key,
                &preview,
                &[((&ca_cert).try_into().unwrap())],
            )
            .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_public_key_mismatch_error() {
        let (credential_response, preview, ca_cert, _) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = *SigningKey::random(&mut OsRng).verifying_key();
        let error = credential_response
            .into_credential::<SoftwareEcdsaKey>(
                "key_id".to_string(),
                &other_public_key,
                &preview,
                &[((&ca_cert).try_into().unwrap())],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::PublicKeyMismatch)
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credential_response, preview, ca_cert, mdoc_public_key) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` from a response
        // that contains insufficient random data should fail.
        let credential_response = match credential_response {
            CredentialResponse::MsoMdoc { mut credential } => {
                let CborBase64(ref mut credential_inner) = credential;
                let name_spaces = credential_inner.name_spaces.as_mut().unwrap();

                name_spaces.modify_first_attributes(|attributes| {
                    let TaggedBytes(first_item) = attributes.first_mut().unwrap();

                    first_item.random = ByteBuf::from(b"12345");
                });

                CredentialResponse::MsoMdoc { credential }
            }
            CredentialResponse::Jwt { credential: _ } => panic!("unexpected credential format"),
        };

        let error = credential_response
            .into_credential::<SoftwareEcdsaKey>(
                "key_id".to_string(),
                &mdoc_public_key,
                &preview,
                &[((&ca_cert).try_into().unwrap())],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH)
        )
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error() {
        let (credential_response, preview, ca_cert, mdoc_public_key) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let other_issuance_key = other_ca
            .generate_issuer_mock(IssuerRegistration::new_mock().into())
            .unwrap();
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer: _,
            } => CredentialPreview::MsoMdoc {
                unsigned_mdoc,
                issuer: other_issuance_key.certificate().clone(),
            },
            _ => panic!("unexpected credential format"),
        };

        let error = credential_response
            .into_credential::<SoftwareEcdsaKey>(
                "key_id".to_string(),
                &mdoc_public_key,
                &preview,
                &[((&ca_cert).try_into().unwrap())],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::IssuerMismatch)
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credential_response, preview, _, mdoc_public_key) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credential_response
            .into_credential::<SoftwareEcdsaKey>("key_id".to_string(), &mdoc_public_key, &preview, &[])
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, IssuanceSessionError::MdocVerification(_))
    }

    #[tokio::test]
    async fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credential_response, preview, ca_cert, mdoc_public_key) = create_credential_response().await;

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail.
        let preview = match preview {
            CredentialPreview::MsoMdoc {
                unsigned_mdoc: _,
                issuer,
            } => CredentialPreview::MsoMdoc {
                unsigned_mdoc: UnsignedMdoc::from(data::pid_full_name().into_first().unwrap()),
                issuer,
            },
            _ => panic!("unexpected credential format"),
        };

        let error = credential_response
            .into_credential::<SoftwareEcdsaKey>(
                "key_id".to_string(),
                &mdoc_public_key,
                &preview,
                &[((&ca_cert).try_into().unwrap())],
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(
            error,
            IssuanceSessionError::IssuedMdocAttributesMismatch(IssuedAttributesMismatch { missing, unexpected })
                if missing.len() == 1 && unexpected.is_empty()
        )
    }
}
