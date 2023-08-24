use std::time::Duration;

use async_trait::async_trait;
use futures::future::TryFutureExt;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::RequestKeyGenerationMessage,
    holder::{self, IssuanceUserConsent, TrustAnchor, Wallet as MdocWallet},
    utils::mdocs_map::MdocsMap,
    ServiceEngagement,
};
use wallet_common::keys::software::SoftwareEcdsaKey;

use super::{PidRetriever, PidRetrieverError};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct PidIssuerClient {
    http_client: reqwest::Client,
}

impl PidIssuerClient {
    fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");

        PidIssuerClient { http_client }
    }
}

impl Default for PidIssuerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PidRetriever for PidIssuerClient {
    async fn retrieve_pid<'a>(
        &self,
        base_url: &Url,
        mdoc_trust_anchors: &[TrustAnchor<'a>],
        access_token: &str,
    ) -> Result<(), PidRetrieverError> {
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

        let mdocs = MdocWallet::new(MdocsMap::new());
        mdocs
            .do_issuance::<SoftwareEcdsaKey>(
                service_engagement,
                &always_agree(),
                &holder::cbor_http_client_builder(),
                mdoc_trust_anchors,
            )
            .await?;

        Ok(())
    }
}

fn always_agree() -> impl IssuanceUserConsent {
    struct AlwaysAgree;
    #[async_trait]
    impl IssuanceUserConsent for AlwaysAgree {
        async fn ask(&self, _: &RequestKeyGenerationMessage) -> bool {
            true
        }
    }
    AlwaysAgree
}
