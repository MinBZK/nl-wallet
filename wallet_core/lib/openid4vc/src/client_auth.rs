use error_category::ErrorCategory;
use http::header::ACCEPT;
use http_utils::reqwest::HttpClient;
use jwt::nonce::Nonce;
use jwt::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ClientAttestationError {
    #[error("cannot use both a challenge endpoint and a header value for attestation-based client authentication")]
    #[category(unexpected)]
    DoubleChallengeMechanism,

    #[error("could not request WIA challenge: {0}")]
    #[category(pd)]
    ChallengeRequest(#[source] reqwest::Error),

    #[error("WIA challenge endpoint returned an error status: {0}")]
    #[category(pd)]
    ChallengeStatus(#[source] reqwest::Error),

    #[error("could not parse WIA challenge response: {0}")]
    #[category(pd)]
    ChallengeBody(#[source] reqwest::Error),

    #[error("OAuth-Client-Attestation-Challenge contained non-UTF8 bytes")]
    #[category(critical)]
    ChallengeHeaderNonUtf8Bytes,
}

/// The Attestation-Based Client Authentication challenge mechanism to be used during the Token Request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientAttestationChallengeMechanism {
    ChallengeEndpoint(Url),
    Header(Nonce),
    None,
}

impl ClientAttestationChallengeMechanism {
    pub fn try_new(endpoint: Option<Url>, http_response: &Response) -> Result<Self, ClientAttestationError> {
        let header_value = http_response
            .headers()
            .get(WIA_CLIENT_CHALLENGE_HEADER_NAME)
            .map(|value| {
                Ok::<_, ClientAttestationError>(Nonce::from(
                    value
                        .to_str()
                        .map_err(|_| ClientAttestationError::ChallengeHeaderNonUtf8Bytes)?
                        .to_string(),
                ))
            })
            .transpose()?;

        match (endpoint, header_value) {
            (Some(_), Some(_)) => Err(ClientAttestationError::DoubleChallengeMechanism),
            (None, None) => Ok(Self::None),
            (None, Some(challenge)) => Ok(Self::Header(challenge)),
            (Some(url), None) => Ok(Self::ChallengeEndpoint(url)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttestationChallenge {
    pub attestation_challenge: Nonce,
}

pub async fn fetch_client_auth_challenge(
    http_client: &HttpClient,
    challenge_endpoint: Url,
) -> Result<Nonce, ClientAttestationError> {
    let challenge = http_client
        .post(challenge_endpoint, |builder| builder.header(ACCEPT, "application/json"))
        .await
        .map_err(ClientAttestationError::ChallengeRequest)?
        .error_for_status()
        .map_err(ClientAttestationError::ChallengeStatus)?
        .json::<AttestationChallenge>()
        .await
        .map_err(ClientAttestationError::ChallengeBody)?
        .attestation_challenge;

    Ok(challenge)
}
