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
    utils::keys::{KeyFactory, MdocEcdsaKey},
};
use wallet_common::generator::TimeGenerator;

use crate::{
    credential::{
        CredentialErrorType, CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponse,
        CredentialResponses,
    },
    dpop::{Dpop, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    token::{AttestationPreview, TokenErrorType, TokenRequest, TokenResponseWithPreviews},
    ErrorResponse, Format, IssuerClientError, NL_WALLET_CLIENT_ID,
};

pub trait IssuerClient {
    fn has_session(&self) -> bool;

    async fn start_issuance(
        &mut self,
        base_url: &Url,
        token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, IssuerClientError>;

    async fn accept_issuance<K: MdocEcdsaKey>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, IssuerClientError>;

    async fn reject_issuance(&mut self) -> Result<(), IssuerClientError>;
}

pub struct HttpIssuerClient {
    http_client: reqwest::Client,
    session_state: Option<IssuanceState>,
}

struct IssuanceState {
    access_token: String,
    c_nonce: String,
    attestation_previews: Vec<AttestationPreview>,
    issuer_url: Url,
    dpop_private_key: SigningKey,
    dpop_nonce: Option<String>,
}

impl HttpIssuerClient {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            session_state: None,
        }
    }
}

impl IssuerClient for HttpIssuerClient {
    fn has_session(&self) -> bool {
        self.session_state.is_some()
    }

    async fn start_issuance(
        &mut self,
        base_url: &Url,
        token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, IssuerClientError> {
        let url = base_url.join("token").unwrap();

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, url.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = self
            .http_client
            .post(url) // TODO discover token endpoint instead
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
            .await?;

        self.session_state = Some(IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response
                .token_response
                .c_nonce
                .ok_or(IssuerClientError::MissingNonce)?,
            attestation_previews: token_response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
            dpop_private_key,
            dpop_nonce,
        });

        Ok(token_response.attestation_previews)
    }

    async fn accept_issuance<K: MdocEcdsaKey>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, IssuerClientError> {
        let issuance_state = self
            .session_state
            .as_ref()
            .ok_or(IssuerClientError::MissingIssuanceSessionState)?;

        // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
        // has no support (yet) for issuance of multiple copies of multiple attestations.
        // We implement this below by simply flattening the relevant nested iterators when communicating with the issuer.

        let doctypes = issuance_state
            .attestation_previews
            .iter()
            .flat_map(|preview| {
                itertools::repeat_n(
                    match preview {
                        AttestationPreview::MsoMdoc { unsigned_mdoc } => unsigned_mdoc.doc_type.clone(),
                    },
                    preview.copy_count() as usize,
                )
            })
            .collect_vec();

        // Generate the PoPs to be sent to the issuer, and the private keys with which they were generated
        // (i.e., the private key of the future mdoc).
        // If N is the total amount of copies of attestations to be issued, then this returns N key/proof pairs.
        let keys_and_proofs = CredentialRequestProof::new_multiple(
            issuance_state.c_nonce.clone(),
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

        let url = issuance_state.issuer_url.join("batch_credential").unwrap();
        let (dpop_header, access_token_header) = issuance_state.auth_headers(url.clone(), Method::POST).await?;

        let responses: CredentialResponses = self
            .http_client
            .post(url) // TODO discover token endpoint instead
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .json(&CredentialRequests { credential_requests })
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
            .await?;

        // The server must have responded with enough credential responses so that we have exactly enough responses
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
        let responses_and_keys: Vec<_> = responses.credential_responses.into_iter().zip(keys).collect();

        // Group all the responses and keys (a flat `Vec`) back into a nested structure,
        // i.e., with the responses/keys grouped per the attestation for which they are intended.
        let grouped_responses_and_keys = issuance_state
            .attestation_previews
            .iter()
            .enumerate()
            .flat_map(|(idx, preview)| itertools::repeat_n(idx, preview.copy_count() as usize))
            .zip(responses_and_keys)
            .group_by(|(idx, _)| *idx);

        // Finally we can iterate over the attestation previews, and using all responses/keys received for each of them
        // turn them into mdoc copies.
        let mdocs = grouped_responses_and_keys
            .into_iter()
            .zip(&issuance_state.attestation_previews)
            .map(|((_, responses_and_keys), preview)| {
                let cred_copies = responses_and_keys
                    .map(move |(_, (cred_response, (pubkey, key_id)))| {
                        cred_response.into_mdoc::<K>(key_id, pubkey, preview.into(), trust_anchors)
                    })
                    .collect::<Result<_, _>>()?;
                Ok(MdocCopies { cred_copies })
            })
            .collect::<Result<_, IssuerClientError>>()?;

        // Clear session state now that all fallible operations have not failed
        self.session_state.take();

        Ok(mdocs)
    }

    async fn reject_issuance(&mut self) -> Result<(), IssuerClientError> {
        let issuance_state = self
            .session_state
            .take()
            .ok_or(IssuerClientError::MissingIssuanceSessionState)?;
        let url = issuance_state.issuer_url.join("batch_credential").unwrap();
        let (dpop_header, access_token_header) = issuance_state.auth_headers(url.clone(), Method::DELETE).await?;

        self.http_client
            .delete(url) // TODO discover token endpoint instead
            .header(DPOP_HEADER_NAME, dpop_header)
            .header(AUTHORIZATION, access_token_header)
            .send()
            .await?
            .error_for_status()?;

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
            Some(self.access_token.clone()),
            self.dpop_nonce.clone(),
        )
        .await?;

        let access_token_header = "DPoP ".to_string() + &self.access_token;

        Ok((dpop_header.as_ref().to_string(), access_token_header))
    }
}
