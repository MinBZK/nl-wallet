use async_trait::async_trait;
use base64::prelude::*;
use futures::future::TryFutureExt;
use http::{
    header::{self, AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use openid4vc::{
    credential::{CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponses},
    token::{TokenErrorResponse, TokenResponseWithPreviews},
};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{CborHttpClient, Mdoc, MdocCopies, TrustAnchor, Wallet as MdocWallet},
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::cbor_deserialize,
    },
    IssuerSigned, ServiceEngagement,
};
use wallet_common::generator::TimeGenerator;

use crate::{digid::DigidSession, utils::reqwest::default_reqwest_client_builder};

use super::{PidIssuerClient, PidIssuerError};

pub struct HttpPidIssuerClient {
    http_client: reqwest::Client,
    mdoc_wallet: MdocWallet,
    issuance_state: Option<Openid4vciIssuanceState>,
}

struct Openid4vciIssuanceState {
    access_token: String,
    c_nonce: String,
    unsigned_mdocs: Vec<UnsignedMdoc>,
    issuer_url: Url,
}

impl HttpPidIssuerClient {
    pub fn new(mdoc_wallet: MdocWallet) -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        HttpPidIssuerClient {
            http_client,
            mdoc_wallet,
            issuance_state: None,
        }
    }
}

impl Default for HttpPidIssuerClient {
    fn default() -> Self {
        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        Self::new(MdocWallet::new(CborHttpClient(http_client)))
    }
}

#[async_trait]
impl PidIssuerClient for HttpPidIssuerClient {
    fn has_session(&self) -> bool {
        self.mdoc_wallet.has_issuance_session()
    }

    async fn start_openid4vci_retrieve_pid<DGS: DigidSession + Send + Sync>(
        &mut self,
        digid_session: DGS,
        base_url: &Url,
        pre_authorized_code: String,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        let token_request = digid_session.into_pre_authorized_code_request(pre_authorized_code);

        let client = default_reqwest_client_builder().build().unwrap(); // TODO

        let token_response: serde_json::Value = client
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

        self.issuance_state = Some(Openid4vciIssuanceState {
            access_token: response.token_response.access_token,
            c_nonce: response.token_response.c_nonce.expect("missing c_nonce"), // TODO
            unsigned_mdocs: response.attestation_previews.clone(),
            issuer_url: base_url.clone(),
        });

        Ok(response.attestation_previews)
    }

    async fn start_retrieve_pid(
        &mut self,
        base_url: &Url,
        access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        let url = base_url
            .join("start")
            .expect("Could not create \"start\" URL from PID issuer base URL");

        let service_engagement = self
            .http_client
            .post(url)
            .bearer_auth(access_token)
            .send()
            .map_err(PidIssuerError::from)
            .and_then(|response| async {
                // Try to get the body from any 4xx or 5xx error responses,
                // in order to create an Error::PidIssuerResponse.
                // TODO: Implement proper JSON-based error reporting
                //       for the mock PID issuer.
                match response.error_for_status_ref() {
                    Ok(_) => Ok(response),
                    Err(error) => {
                        let error = match response.text().await.ok() {
                            Some(body) => PidIssuerError::Response(error, body),
                            None => PidIssuerError::Networking(error),
                        };

                        Err(error)
                    }
                }
            })
            .await?
            .json::<ServiceEngagement>()
            .await?;

        let unsigned_mdocs = self.mdoc_wallet.start_issuance(service_engagement).await?;

        Ok(unsigned_mdocs.to_vec())
    }

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        let mdocs = self
            .mdoc_wallet
            .finish_issuance(mdoc_trust_anchors, key_factory)
            .await?;

        Ok(mdocs)
    }

    async fn accept_openid4vci_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        wallet_name: String,
        audience: String,
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        let issuance_state = self.issuance_state.take().expect("no issuance state");
        let keys_count: u64 = issuance_state
            .unsigned_mdocs
            .iter()
            .map(|unsigned| unsigned.copy_count)
            .sum();

        let keys_and_responses = CredentialRequestProof::new_multiple(
            issuance_state.c_nonce.clone(),
            wallet_name,
            audience,
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
                        format: openid4vc::Format::MsoMdoc,
                        proof: response,
                    },
                )
            })
            .unzip();

        let client = default_reqwest_client_builder().build().unwrap(); // TODO

        let credential_requests = CredentialRequests {
            credential_requests: responses,
        };
        let responses: CredentialResponses = client
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

        Ok(mdocs)
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        self.mdoc_wallet.stop_issuance().await?;

        Ok(())
    }
}
