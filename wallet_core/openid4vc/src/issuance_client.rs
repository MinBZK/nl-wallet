use base64::prelude::*;
use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
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
    credential::{CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponses},
    token::{TokenErrorResponse, TokenRequest, TokenResponseWithPreviews},
    Error, Format, NL_WALLET_CLIENT_ID,
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
        let token_response: serde_json::Value = self
            .http_client
            .post(base_url.join("/issuance/token").unwrap()) // TODO discover token endpoint instead
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(serde_urlencoded::to_string(token_request).unwrap()) // TODO
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let error: Result<TokenErrorResponse, _> = serde_json::from_value(token_response.clone());
        if let Ok(error) = error {
            panic!("{:?}", error); // TODO
        }

        let response: TokenResponseWithPreviews = serde_json::from_value(token_response).unwrap(); // TODO

        self.session_state = Some(IssuanceState {
            access_token: response.token_response.access_token,
            c_nonce: response.token_response.c_nonce.expect("missing c_nonce"), // TODO
            unsigned_mdocs: response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
        });

        Ok(response.attestation_previews)
    }

    pub async fn finish_issuance<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, Error> {
        let issuance_state = self.session_state.as_ref().expect("no issuance state");

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
        .await
        .unwrap(); // TODO

        let (keys, responses): (Vec<K>, Vec<CredentialRequest>) = keys_and_responses
            .into_iter()
            .map(|(key, response)| {
                (
                    key,
                    CredentialRequest {
                        format: Format::MsoMdoc,
                        proof: response,
                    },
                )
            })
            .unzip();

        let credential_requests = CredentialRequests {
            credential_requests: responses,
        };
        let responses: CredentialResponses = self
            .http_client
            .post(issuance_state.issuer_url.join("/issuance/batch_credential").unwrap()) // TODO discover token endpoint instead
            .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
            .header(AUTHORIZATION, "Bearer ".to_string() + &issuance_state.access_token)
            .body(serde_json::to_string(&credential_requests).unwrap()) // TODO
            .send()
            .await
            .unwrap() // TODO parse as credential error response in case of 4xx or 5xx
            .json()
            .await
            .unwrap();
        let mut keys_and_responses: Vec<_> = responses.credential_responses.into_iter().zip(keys).collect();

        let mdocs = issuance_state
            .unsigned_mdocs
            .iter()
            .map(|unsigned| MdocCopies {
                // TODO check that the received attributes equal the previously received unsigned mdocs
                cred_copies: keys_and_responses
                    .drain(..unsigned.copy_count as usize)
                    .map(|(cred_response, key)| {
                        let issuer_signed: String = serde_json::from_value(cred_response.credential).unwrap();
                        let issuer_signed: IssuerSigned =
                            cbor_deserialize(BASE64_URL_SAFE_NO_PAD.decode(issuer_signed).unwrap().as_slice()).unwrap();

                        // Construct the new mdoc; this also verifies it against the trust anchors.
                        Mdoc::new::<K>(
                            key.identifier().to_string(),
                            issuer_signed,
                            &TimeGenerator,
                            trust_anchors,
                        )
                        .unwrap()
                    })
                    .collect(),
            })
            .collect();

        // Clear session state now that all fallible operations have not failed
        self.session_state.take();

        Ok(mdocs)
    }

    pub async fn stop_issuance(&mut self) -> Result<(), Error> {
        let issuance_state = self.session_state.take().expect("no issuance state");

        self.http_client
            .delete(issuance_state.issuer_url.join("/issuance/reject").unwrap()) // TODO discover token endpoint instead
            .header(AUTHORIZATION, "Bearer ".to_string() + &issuance_state.access_token)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}
