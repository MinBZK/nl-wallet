use std::collections::VecDeque;

use futures::{future::try_join_all, TryFutureExt};
use itertools::Itertools;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use reqwest::{header::AUTHORIZATION, Method};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{Mdoc, MdocCopies, TrustAnchor},
    identifiers::AttributeIdentifier,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::CborError,
    },
};
use wallet_common::{generator::TimeGenerator, jwt::JwtError};

use crate::{
    credential::{
        CredentialErrorType, CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponse,
        CredentialResponses,
    },
    dpop::{Dpop, DpopError, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    jwt::JwkConversionError,
    token::{AccessToken, AttestationPreview, TokenErrorType, TokenRequest, TokenResponseWithPreviews},
    ErrorResponse, Format, NL_WALLET_CLIENT_ID,
};

#[derive(Debug, thiserror::Error)]
pub enum IssuerClientError {
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("DPoP error: {0}")]
    Dpop(#[from] DpopError),
    #[error("failed to convert key from/to JWK format: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
    #[error("http request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("missing c_nonce")]
    MissingNonce,
    #[error("CBOR (de)serialization error: {0}")]
    Cbor(#[from] CborError),
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("mismatch between issued and expected attributes")]
    IssuedAttributesMismatch(Vec<AttributeIdentifier>),
    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] nl_wallet_mdoc::Error),
    #[error("error requesting access token: {0:?}")]
    TokenRequest(Box<ErrorResponse<TokenErrorType>>),
    #[error("error requesting credentials: {0:?}")]
    CredentialRequest(Box<ErrorResponse<CredentialErrorType>>),
    #[error("generating attestation private keys failed: {0}")]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("public key contained in mdoc not equal to expected value")]
    PublicKeyMismatch,
    #[error("failed to get mdoc public key: {0}")]
    PublicKeyFromMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("received {found} responses, expected {expected}")]
    UnexpectedCredentialResponseCount { found: usize, expected: usize },
}

pub trait IssuerClient<H = HttpOpenidMessageClient> {
    async fn start_issuance(
        message_client: H,
        base_url: &Url,
        token_request: TokenRequest,
    ) -> Result<(Self, Vec<AttestationPreview>), IssuerClientError>
    where
        Self: Sized;

    async fn accept_issuance<K: MdocEcdsaKey>(
        self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, IssuerClientError>;

    async fn reject_issuance(self) -> Result<(), IssuerClientError>;
}

pub struct HttpIssuerClient<H = HttpOpenidMessageClient> {
    message_client: H,
    session_state: IssuanceState,
}

/// Contract for sending OpenID4VCI protocol messages.
pub trait OpenidMessageClient {
    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuerClientError>;

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuerClientError>;

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), IssuerClientError>;
}

pub struct HttpOpenidMessageClient {
    http_client: reqwest::Client,
}

impl HttpOpenidMessageClient {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }
}

impl OpenidMessageClient for HttpOpenidMessageClient {
    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuerClientError> {
        self.http_client
            .post(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header.as_ref())
            .form(&token_request)
            .send()
            .map_err(IssuerClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorType>>().await?;
                    Err(IssuerClientError::TokenRequest(error.into()))
                } else {
                    let dpop_nonce = response
                        .headers()
                        .get(DPOP_NONCE_HEADER_NAME)
                        .and_then(|val| val.to_str().map(str::to_string).ok());
                    let deserialized = response.json::<TokenResponseWithPreviews>().await?;
                    Ok((deserialized, dpop_nonce))
                }
            })
            .await
    }

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuerClientError> {
        self.http_client
            .post(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .json(credential_requests)
            .send()
            .map_err(IssuerClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorType>>().await?;
                    Err(IssuerClientError::CredentialRequest(error.into()))
                } else {
                    let text = response.json().await?;
                    Ok(text)
                }
            })
            .await
    }

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), IssuerClientError> {
        self.http_client
            .delete(url.as_ref())
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

struct IssuanceState {
    access_token: AccessToken,
    c_nonce: String,
    attestation_previews: Vec<AttestationPreview>,
    issuer_url: Url,
    dpop_private_key: SigningKey,
    dpop_nonce: Option<String>,
}

impl<H: OpenidMessageClient> IssuerClient<H> for HttpIssuerClient<H> {
    async fn start_issuance(
        message_client: H,
        base_url: &Url,
        token_request: TokenRequest,
    ) -> Result<(Self, Vec<AttestationPreview>), IssuerClientError> {
        let url = base_url.join("token").unwrap(); // TODO discover token endpoint instead

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, url.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = message_client.request_token(&url, &token_request, &dpop_header).await?;

        let session_state = IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response
                .token_response
                .c_nonce
                .ok_or(IssuerClientError::MissingNonce)?,
            attestation_previews: token_response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
            dpop_private_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };
        Ok((issuance_client, token_response.attestation_previews))
    }

    async fn accept_issuance<K: MdocEcdsaKey>(
        self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, IssuerClientError> {
        // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
        // has no support (yet) for issuance of multiple copies of multiple attestations.
        // We implement this below by simply flattening the relevant nested iterators when communicating with the issuer.

        let doctypes = self
            .session_state
            .attestation_previews
            .iter()
            .flat_map(|preview| {
                itertools::repeat_n(
                    match preview {
                        AttestationPreview::MsoMdoc { unsigned_mdoc } => unsigned_mdoc.doc_type.clone(),
                    },
                    preview.copy_count().try_into().unwrap(),
                )
            })
            .collect_vec();

        // Generate the PoPs to be sent to the issuer, and the private keys with which they were generated
        // (i.e., the private key of the future mdoc).
        // If N is the total amount of copies of attestations to be issued, then this returns N key/proof pairs.
        let keys_and_proofs = CredentialRequestProof::new_multiple(
            self.session_state.c_nonce.clone(),
            NL_WALLET_CLIENT_ID.to_string(),
            credential_issuer_identifier,
            doctypes.len() as u64,
            key_factory,
        )
        .await?;

        // Split into N keys and N credential requests, so we can send the credential request proofs separately
        // to the issuer.
        let (keys, credential_requests): (Vec<K>, Vec<CredentialRequest>) = keys_and_proofs
            .into_iter()
            .zip(doctypes)
            .map(|((key, response), doctype)| {
                (
                    key,
                    CredentialRequest {
                        format: Format::MsoMdoc,
                        doctype: Some(doctype),
                        proof: Some(response),
                    },
                )
            })
            .unzip();

        let url = self.session_state.issuer_url.join("batch_credential").unwrap(); // TODO discover token endpoint instead
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::POST).await?;

        let responses = self
            .message_client
            .request_credentials(
                &url,
                &CredentialRequests { credential_requests },
                &dpop_header,
                &access_token_header,
            )
            .await?;

        // The server must have responded with enough credential responses, N, so that we have exactly enough responses
        // for all copies of all mdocs constructed below.
        if responses.credential_responses.len() != keys.len() {
            return Err(IssuerClientError::UnexpectedCredentialResponseCount {
                found: responses.credential_responses.len(),
                expected: keys.len(),
            });
        }

        let keys: Vec<_> = try_join_all(keys.iter().map(|key| async {
            let pubkey = key
                .verifying_key()
                .await
                .map_err(|e| IssuerClientError::VerifyingKeyFromPrivateKey(e.into()))?;
            let id = key.identifier().to_string();
            Ok::<_, IssuerClientError>((pubkey, id))
        }))
        .await?;
        let mut responses_and_keys: VecDeque<_> = responses.credential_responses.into_iter().zip(keys).collect();

        let mdocs = self
            .session_state
            .attestation_previews
            .iter()
            .map(|preview| {
                let copy_count: usize = preview.copy_count().try_into().unwrap();

                // Consume the amount of copies from the front of `responses_and_keys`.
                let cred_copies = responses_and_keys
                    .drain(..copy_count)
                    .map(|(cred_response, (pubkey, key_id))| {
                        // Convert the response into an `Mdoc`, verifying it against both the
                        // trust anchors and the `UnsignedMdoc` we received in the preview.
                        cred_response.into_mdoc::<K>(key_id, pubkey, preview.as_ref(), trust_anchors)
                    })
                    .collect::<Result<_, _>>()?;

                // For each preview we have an `MdocCopies` instance.
                Ok(MdocCopies { cred_copies })
            })
            .collect::<Result<_, IssuerClientError>>()?;

        Ok(mdocs)
    }

    async fn reject_issuance(self) -> Result<(), IssuerClientError> {
        let url = self.session_state.issuer_url.join("batch_credential").unwrap(); // TODO discover token endpoint instead
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), Method::DELETE).await?;

        self.message_client
            .reject(&url, &dpop_header, &access_token_header)
            .await?;

        Ok(())
    }
}

impl CredentialResponse {
    /// Create an [`Mdoc`] out of the credential response. Also verifies the mdoc.
    fn into_mdoc<K: MdocEcdsaKey>(
        self,
        key_id: String,
        verifying_key: VerifyingKey,
        unsigned: &UnsignedMdoc,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Mdoc, IssuerClientError> {
        let issuer_signed = match self {
            CredentialResponse::MsoMdoc { credential } => credential.0,
        };

        if issuer_signed
            .public_key()
            .map_err(IssuerClientError::PublicKeyFromMdoc)?
            != verifying_key
        {
            return Err(IssuerClientError::PublicKeyMismatch);
        }

        // Construct the new mdoc; this also verifies it against the trust anchors.
        let mdoc = Mdoc::new::<K>(key_id, issuer_signed, &TimeGenerator, trust_anchors)
            .map_err(IssuerClientError::MdocVerification)?;

        // Check that our mdoc contains exactly the attributes the issuer said it would have
        mdoc.compare_unsigned(unsigned)
            .map_err(IssuerClientError::IssuedAttributesMismatch)?;

        Ok(mdoc)
    }
}

impl IssuanceState {
    async fn auth_headers(&self, url: Url, method: reqwest::Method) -> Result<(String, String), IssuerClientError> {
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
