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
    Error, ErrorResponse, Format, NL_WALLET_CLIENT_ID,
};

pub struct IssuanceClient {
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

impl IssuanceClient {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            session_state: None,
        }
    }

    pub fn has_issuance_session(&self) -> bool {
        self.session_state.is_some()
    }

    pub async fn start_issuance(
        &mut self,
        base_url: &Url,
        token_request: TokenRequest,
    ) -> Result<Vec<AttestationPreview>, Error> {
        let url = base_url.join("token").unwrap();

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, url.clone(), Method::POST, None, None).await?;

        let (token_response, dpop_nonce) = self
            .http_client
            .post(url) // TODO discover token endpoint instead
            .header(DPOP_HEADER_NAME, dpop_header.as_ref())
            .form(&token_request)
            .send()
            .map_err(Error::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorType>>().await?;
                    Err(Error::TokenRequest(error.into()))
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
            c_nonce: token_response.token_response.c_nonce.ok_or(Error::MissingNonce)?,
            attestation_previews: token_response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
            dpop_private_key,
            dpop_nonce,
        });

        Ok(token_response.attestation_previews)
    }

    pub async fn finish_issuance<K: MdocEcdsaKey>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, Error> {
        let issuance_state = self.session_state.as_ref().ok_or(Error::MissingIssuanceSessionState)?;

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

        let keys_and_proofs = CredentialRequestProof::new_multiple(
            issuance_state.c_nonce.clone(),
            NL_WALLET_CLIENT_ID.to_string(),
            credential_issuer_identifier,
            doctypes.len() as u64,
            key_factory,
        )
        .await?;

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
            .map_err(Error::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorType>>().await?;
                    Err(Error::CredentialRequest(error.into()))
                } else {
                    let text = response.json().await?;
                    Ok(text)
                }
            })
            .await?;

        let keys: Vec<_> = try_join_all(keys.iter().map(|key| async {
            let pubkey = key
                .verifying_key()
                .await
                .map_err(|e| Error::VerifyingKeyFromPrivateKey(e.into()))?;
            let id = key.identifier().to_string();
            Ok::<_, Error>((pubkey, id))
        }))
        .await?;
        let responses_and_keys: Vec<_> = responses.credential_responses.into_iter().zip(keys).collect();

        let mdocs = issuance_state
            .attestation_previews
            .iter()
            .enumerate()
            .flat_map(|(idx, preview)| itertools::repeat_n(idx, preview.copy_count() as usize))
            .zip(responses_and_keys)
            .group_by(|(idx, _)| *idx)
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
            .collect::<Result<_, Error>>()?;

        // Clear session state now that all fallible operations have not failed
        self.session_state.take();

        Ok(mdocs)
    }

    pub async fn stop_issuance(&mut self) -> Result<(), Error> {
        let issuance_state = self.session_state.take().ok_or(Error::MissingIssuanceSessionState)?;
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
    ) -> Result<Mdoc, Error> {
        let issuer_signed = match self {
            CredentialResponse::MsoMdoc { credential } => credential.0,
        };

        if issuer_signed.public_key().map_err(Error::PublicKeyFromMdoc)? != verifying_key {
            return Err(Error::PublicKeyMismatch);
        }

        // Construct the new mdoc; this also verifies it against the trust anchors.
        let mdoc =
            Mdoc::new::<K>(key_id, issuer_signed, &TimeGenerator, trust_anchors).map_err(Error::MdocVerification)?;

        // Check that our mdoc contains exactly the attributes the issuer said it would have
        mdoc.compare_unsigned(unsigned)
            .map_err(Error::IssuedAttributesMismatch)?;

        Ok(mdoc)
    }
}

impl IssuanceState {
    async fn auth_headers(&self, url: Url, method: reqwest::Method) -> Result<(String, String), Error> {
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
