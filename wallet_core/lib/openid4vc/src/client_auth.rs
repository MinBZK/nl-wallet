use http::header::ACCEPT;
use http_utils::reqwest::HttpClient;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::wallet_issuance::WalletIssuanceError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttestationChallenge {
    pub attestation_challenge: Nonce,
}

pub async fn fetch_client_auth_challenge(
    http_client: &HttpClient,
    challenge_endpoint: Option<Url>,
) -> Result<Option<Nonce>, WalletIssuanceError> {
    if let Some(challenge_endpoint) = challenge_endpoint {
        let challenge = http_client
            .post(challenge_endpoint, |builder| builder.header(ACCEPT, "application/json"))
            .await
            .map_err(WalletIssuanceError::WiaChallenge)?
            .error_for_status()
            .map_err(WalletIssuanceError::WiaChallenge)?
            .json::<AttestationChallenge>()
            .await
            .map_err(WalletIssuanceError::WiaChallenge)?
            .attestation_challenge;

        Ok(Some(challenge))
    } else {
        Ok(None)
    }
}
