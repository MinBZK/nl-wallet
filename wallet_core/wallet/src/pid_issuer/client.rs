use async_trait::async_trait;
use futures::future::TryFutureExt;
use http::{header, HeaderMap, HeaderValue};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{CborHttpClient, TrustAnchor, Wallet as MdocWallet},
    utils::{keys::KeyFactory, mdocs_map::MdocsMap},
    ServiceEngagement,
};

use crate::utils::reqwest::default_reqwest_client_builder;

use super::{PidRetriever, PidRetrieverError};

// TODO: The `mdoc_wallet` field uses `Arc<>` just for testing now.
//       This should be removed as soon as actual storage is implemented.
pub struct PidIssuerClient {
    http_client: reqwest::Client,
    mdoc_wallet: Arc<Mutex<MdocWallet<MdocsMap>>>,
}

impl PidIssuerClient {
    pub fn new(mdoc_wallet: Arc<Mutex<MdocWallet<MdocsMap>>>) -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        PidIssuerClient {
            http_client,
            mdoc_wallet,
        }
    }
}

impl Default for PidIssuerClient {
    fn default() -> Self {
        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        Self::new(Arc::new(Mutex::new(MdocWallet::new(
            MdocsMap::new(),
            CborHttpClient(http_client),
        ))))
    }
}

#[async_trait]
impl PidRetriever for PidIssuerClient {
    async fn start_retrieve_pid(
        &mut self,
        base_url: &Url,
        access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidRetrieverError> {
        let url = base_url
            .join("start")
            .expect("Could not create \"start\" URL from PID issuer base URL");

        let service_engagement = self
            .http_client
            .post(url)
            .bearer_auth(access_token)
            .send()
            .map_err(PidRetrieverError::from)
            .and_then(|response| async {
                // Try to get the body from any 4xx or 5xx error responses,
                // in order to create an Error::PidIssuerResponse.
                // TODO: Implement proper JSON-based error reporting
                //       for the mock PID issuer.
                match response.error_for_status_ref() {
                    Ok(_) => Ok(response),
                    Err(error) => {
                        let error = match response.text().await.ok() {
                            Some(body) => PidRetrieverError::PidIssuerResponse(error, body),
                            None => PidRetrieverError::PidIssuer(error),
                        };

                        Err(error)
                    }
                }
            })
            .await?
            .json::<ServiceEngagement>()
            .await?;

        let mut mdoc_wallet = self.mdoc_wallet.lock().await;
        let unsigned_mdocs = mdoc_wallet.start_issuance(service_engagement).await?;

        Ok(unsigned_mdocs.to_vec())
    }

    async fn accept_pid<'a>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &'a (impl KeyFactory<'a> + Sync),
    ) -> Result<(), PidRetrieverError> {
        let mut mdoc_wallet = self.mdoc_wallet.lock().await;

        mdoc_wallet.finish_issuance(mdoc_trust_anchors, key_factory).await?;

        Ok(())
    }

    async fn reject_pid_issuance(&mut self) -> Result<(), PidRetrieverError> {
        let mut mdoc_wallet = self.mdoc_wallet.lock().await;

        mdoc_wallet.stop_issuance().await?;

        Ok(())
    }
}
