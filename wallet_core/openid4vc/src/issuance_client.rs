use base64::prelude::*;
use futures::{future::try_join_all, TryFutureExt};
use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{Mdoc, MdocCopies, TrustAnchor},
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::cbor_deserialize,
    },
    IssuerSigned,
};
use wallet_common::generator::TimeGenerator;

use crate::{
    credential::{
        CredentialErrorType, CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponse,
        CredentialResponses,
    },
    dpop::Dpop,
    token::{TokenErrorType, TokenRequest, TokenResponseWithPreviews},
    Error, ErrorResponse, Format, NL_WALLET_CLIENT_ID,
};

pub struct IssuanceClient {
    http_client: reqwest::Client,
    session_state: Option<IssuanceState>,
}

struct IssuanceState {
    access_token: String,
    c_nonce: String,
    unsigned_mdocs: Vec<UnsignedMdoc>,
    issuer_url: Url,
    dpop_private_key: SigningKey,
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
    ) -> Result<Vec<UnsignedMdoc>, Error> {
        let url = base_url.join("token").unwrap();

        let dpop_private_key = SigningKey::random(&mut OsRng);
        let dpop_header = Dpop::new(&dpop_private_key, url.clone(), Method::POST, None).await?;

        let token_response: TokenResponseWithPreviews = self
            .http_client
            .post(url) // TODO discover token endpoint instead
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .header("DPoP", dpop_header.0 .0)
            .body(serde_urlencoded::to_string(token_request)?)
            .send()
            .map_err(Error::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<TokenErrorType>>().await?;
                    Err(Error::TokenRequest(error))
                } else {
                    let text = response.json().await?;
                    Ok(text)
                }
            })
            .await?;

        self.session_state = Some(IssuanceState {
            access_token: token_response.token_response.access_token,
            c_nonce: token_response.token_response.c_nonce.ok_or(Error::MissingNonce)?,
            unsigned_mdocs: token_response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
            dpop_private_key,
        });

        Ok(token_response.attestation_previews)
    }

    pub async fn finish_issuance<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, Error> {
        let issuance_state = self.session_state.as_ref().ok_or(Error::MissingIssuanceSessionState)?;

        let keys_count: u64 = issuance_state
            .unsigned_mdocs
            .iter()
            .map(|unsigned| unsigned.copy_count)
            .sum();

        let keys_and_responses = CredentialRequestProof::new_multiple(
            issuance_state.c_nonce.clone(),
            NL_WALLET_CLIENT_ID.to_string(),
            credential_issuer_identifier,
            keys_count,
            key_factory,
        )
        .await?;

        let doctypes = issuance_state
            .unsigned_mdocs
            .iter()
            .flat_map(|unsigned| std::iter::repeat(unsigned.doc_type.clone()).take(unsigned.copy_count as usize));

        let (keys, responses): (Vec<K>, Vec<CredentialRequest>) = keys_and_responses
            .into_iter()
            .zip(doctypes)
            .map(|((key, response), doctype)| {
                (
                    key,
                    CredentialRequest {
                        format: Format::MsoMdoc,
                        doctype: Some(doctype),
                        proof: response,
                    },
                )
            })
            .unzip();

        let url = issuance_state.issuer_url.join("batch_credential").unwrap();
        let dpop_header = Dpop::new(
            &issuance_state.dpop_private_key,
            url.clone(),
            Method::POST,
            Some(issuance_state.access_token.clone()),
        )
        .await?;

        let credential_requests = CredentialRequests {
            credential_requests: responses,
        };
        let responses: CredentialResponses = self
            .http_client
            .post(url) // TODO discover token endpoint instead
            .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
            .header("DPoP", dpop_header.0 .0)
            .header(AUTHORIZATION, "DPoP ".to_string() + &issuance_state.access_token)
            .body(serde_json::to_string(&credential_requests)?)
            .send()
            .map_err(Error::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<CredentialErrorType>>().await?;
                    Err(Error::CredentialRequest(error))
                } else {
                    let text = response.json().await?;
                    Ok(text)
                }
            })
            .await?;

        let mut keys_and_responses: Vec<_> = responses.credential_responses.into_iter().zip(keys).collect();

        let mdocs = try_join_all(
            issuance_state
                .unsigned_mdocs
                .iter()
                // We map in two steps to prevent `keys_and_responses` from appearing in an async closure
                .map(|unsigned| {
                    (
                        unsigned,
                        keys_and_responses
                            .drain(..unsigned.copy_count as usize)
                            .collect::<Vec<_>>(),
                    )
                })
                .map(|(unsigned, keys_and_responses)| async {
                    let copies = MdocCopies {
                        cred_copies: try_join_all(
                            keys_and_responses
                                .into_iter()
                                .map(|(cred_response, key)| cred_response.into_mdoc(key, unsigned, trust_anchors)),
                        )
                        .await?,
                    };
                    Result::<_, Error>::Ok(copies)
                }),
        )
        .await?;

        // Clear session state now that all fallible operations have not failed
        self.session_state.take();

        Ok(mdocs)
    }

    pub async fn stop_issuance(&mut self) -> Result<(), Error> {
        let issuance_state = self.session_state.as_ref().ok_or(Error::MissingIssuanceSessionState)?;

        self.http_client
            .delete(issuance_state.issuer_url.join("credential").unwrap()) // TODO discover token endpoint instead
            .header(AUTHORIZATION, "DPoP ".to_string() + &issuance_state.access_token)
            .send()
            .await?;

        Ok(())
    }
}

impl CredentialResponse {
    /// Create an [`Mdoc`] out of the credential response. Also verifies the mdoc.
    async fn into_mdoc<K: MdocEcdsaKey>(
        self,
        key: K,
        unsigned: &UnsignedMdoc,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Mdoc, Error> {
        let issuer_signed: String = serde_json::from_value(self.credential)?;
        let issuer_signed: IssuerSigned = cbor_deserialize(BASE64_URL_SAFE_NO_PAD.decode(issuer_signed)?.as_slice())?;

        if issuer_signed.public_key().map_err(Error::PublicKeyFromMdoc)?
            != key
                .verifying_key()
                .await
                .map_err(|e| Error::VerifyingKeyFromPrivateKey(e.into()))?
        {
            return Err(Error::PublicKeyMismatch);
        }

        // Construct the new mdoc; this also verifies it against the trust anchors.
        let mdoc = Mdoc::new::<K>(
            key.identifier().to_string(),
            issuer_signed,
            &TimeGenerator,
            trust_anchors,
        )
        .map_err(Error::MdocVerification)?;

        // Check that our mdoc contains exactly the attributes the issuer said it would have
        mdoc.compare_unsigned(unsigned)
            .map_err(|_| Error::ExpectedAttributesMissing)?;

        Ok(mdoc)
    }
}
