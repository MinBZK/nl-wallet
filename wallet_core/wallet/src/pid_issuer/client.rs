use async_trait::async_trait;
use futures::future::TryFutureExt;
use http::{
    header::{self},
    HeaderMap, HeaderValue,
};

use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{CborHttpClient, MdocCopies, TrustAnchor, Wallet as MdocWallet},
    utils::keys::{KeyFactory, MdocEcdsaKey},
    ServiceEngagement,
};
use openid4vc::issuance_client::IssuanceClient;

use crate::{digid::DigidSession, utils::reqwest::default_reqwest_client_builder};

use super::{OpenidPidIssuerClient, PidIssuerClient, PidIssuerError};

pub struct HttpPidIssuerClient {
    http_client: reqwest::Client,
    mdoc_wallet: MdocWallet,
}

pub struct HttpOpenidPidIssuerClient {
    issuance_client: IssuanceClient,
}

impl Default for HttpOpenidPidIssuerClient {
    fn default() -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        HttpOpenidPidIssuerClient {
            issuance_client: IssuanceClient::new(http_client),
        }
    }
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
impl OpenidPidIssuerClient for HttpOpenidPidIssuerClient {
    fn has_session(&self) -> bool {
        self.issuance_client.has_issuance_session()
    }

    async fn start_retrieve_pid<DGS: DigidSession + Send + Sync>(
        &mut self,
        digid_session: DGS,
        base_url: &Url,
        pre_authorized_code: String,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError> {
        let token_request = digid_session.into_pre_authorized_code_request(pre_authorized_code);
        let attestation_previews = self
            .issuance_client
            .start_issuance(base_url, token_request)
            .await
            .unwrap();
        Ok(attestation_previews)
    }

    async fn accept_pid<'a, K: MdocEcdsaKey + Send + Sync>(
        &mut self,
        trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a, Key = K> + Sync),
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, PidIssuerError> {
        let mdocs = self
            .issuance_client
            .finish_issuance(trust_anchors, key_factory, credential_issuer_identifier)
            .await
            .unwrap();

        Ok(mdocs)
    }

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        self.issuance_client.stop_issuance().await.unwrap();

        Ok(())
    }
}

#[async_trait]
impl PidIssuerClient for HttpPidIssuerClient {
    fn has_session(&self) -> bool {
        self.mdoc_wallet.has_issuance_session()
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

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError> {
        self.mdoc_wallet.stop_issuance().await?;

        Ok(())
    }
}
