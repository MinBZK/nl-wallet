use error_category::ErrorCategory;
use http::header::ACCEPT;
use http_utils::reqwest::HttpClient;
use indexmap::IndexSet;
use itertools::Itertools;
use jwt::nonce::Nonce;
use jwt::wia::WIA_CLIENT_AUTH_METHOD;
use jwt::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::jose::JwsAlgorithm;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

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

    #[error("OAuth-Client-Attestation-Challenge contained non-visible-ASCII bytes")]
    #[category(critical)]
    ChallengeHeaderNonVisibleAscii,

    #[error("the Authorization Server does not support attestation-based client authentication")]
    #[category(expected)]
    NoAttestationBasedClientAuthSupport,

    #[error(
        "the Authorization Server does not support ES256 for client attestation signing: {}",
        .0.as_ref().map(|algs| algs.iter().join(", ")).unwrap_or_else(|| "<none>".to_string())
    )]
    #[category(expected)]
    ClientAttestationSigningAlgNotSupported(Option<IndexSet<JwsAlgorithm>>),

    #[error(
        "the Authorization Server does not support ES256 for client attestation PoP signing: {}",
        .0.as_ref().map(|algs| algs.iter().join(", ")).unwrap_or_else(|| "<none>".to_string())
    )]
    #[category(expected)]
    ClientAttestationPopSigningAlgNotSupported(Option<IndexSet<JwsAlgorithm>>),
}

/// Verify that the Authorization Server metadata advertises support for Attestation-Based Client Authentication.
pub fn check_client_attestation_metadata(
    oauth_metadata: &AuthorizationServerMetadata,
) -> Result<(), ClientAttestationError> {
    if !oauth_metadata
        .token_endpoint_auth_methods_supported
        .as_ref()
        .is_some_and(|auth_methods| auth_methods.contains(WIA_CLIENT_AUTH_METHOD))
    {
        return Err(ClientAttestationError::NoAttestationBasedClientAuthSupport);
    }

    if !oauth_metadata
        .client_attestation_signing_alg_values_supported
        .as_ref()
        .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
    {
        return Err(ClientAttestationError::ClientAttestationSigningAlgNotSupported(
            oauth_metadata.client_attestation_signing_alg_values_supported.clone(),
        ));
    }

    if !oauth_metadata
        .client_attestation_pop_signing_alg_values_supported
        .as_ref()
        .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
    {
        return Err(ClientAttestationError::ClientAttestationPopSigningAlgNotSupported(
            oauth_metadata
                .client_attestation_pop_signing_alg_values_supported
                .clone(),
        ));
    }

    Ok(())
}

/// The Attestation-Based Client Authentication challenge mechanism to be used during the Token Request.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAttestationChallengeMechanism {
    ChallengeEndpoint(Url),
    Header(Nonce),
    None,
}

impl ClientAttestationChallengeMechanism {
    /// Construct a new `ClientAttestationChallengeMechanism` during the Authorization Code Flow (ACF).
    pub fn try_new_acf(
        challenge_endpoint: Option<Url>,
        http_response: &Response,
    ) -> Result<Self, ClientAttestationError> {
        let header_value = http_response
            .headers()
            .get(WIA_CLIENT_CHALLENGE_HEADER_NAME)
            .map(|value| {
                Ok::<_, ClientAttestationError>(Nonce::from(
                    value
                        .to_str()
                        .map_err(|_| ClientAttestationError::ChallengeHeaderNonVisibleAscii)?
                        .to_string(),
                ))
            })
            .transpose()?;

        match (challenge_endpoint, header_value) {
            (Some(_), Some(_)) => Err(ClientAttestationError::DoubleChallengeMechanism),
            (None, None) => Ok(Self::None),
            (None, Some(challenge)) => Ok(Self::Header(challenge)),
            (Some(url), None) => Ok(Self::ChallengeEndpoint(url)),
        }
    }

    /// Construct a new `ClientAttestationChallengeMechanism` during the Pre-Authorized Code Flow.
    ///
    /// In the Pre-Authorized Code Flow, no PAR request was sent whose response might have included a
    /// challenge for Attestation-Based Client Authentication. So we can either use the challenge_endpoint,
    /// if the issuer has one, or the issuer does not use WIA PoP challenges so we don't use one.
    pub fn new_pre_authorized(challenge_endpoint: Option<Url>) -> Self {
        match challenge_endpoint {
            None => ClientAttestationChallengeMechanism::None,
            Some(url) => ClientAttestationChallengeMechanism::ChallengeEndpoint(url),
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
        .post(challenge_endpoint, |builder| {
            builder.header(ACCEPT, mime::APPLICATION_JSON.as_ref())
        })
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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use http::Response as HttpResponse;
    use jwt::nonce::Nonce;
    use jwt::wia::WIA_CLIENT_AUTH_METHOD;
    use jwt::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;
    use reqwest::Response;
    use url::Url;

    use super::ClientAttestationChallengeMechanism;
    use super::ClientAttestationError;
    use super::JwsAlgorithm;
    use super::check_client_attestation_metadata;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

    fn http_response(header_value: Option<&str>) -> Response {
        let mut builder = HttpResponse::builder();
        if let Some(header_value) = header_value {
            builder = builder.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, header_value);
        }

        Response::from(builder.body("").unwrap())
    }

    #[test]
    fn try_new_acf_none() {
        let mechanism = ClientAttestationChallengeMechanism::try_new_acf(None, &http_response(None)).unwrap();

        assert_matches!(mechanism, ClientAttestationChallengeMechanism::None);
    }

    #[test]
    fn try_new_acf_challenge_endpoint() {
        let url: Url = "https://issuer.example.com/challenge".parse().unwrap();

        let mechanism =
            ClientAttestationChallengeMechanism::try_new_acf(Some(url.clone()), &http_response(None)).unwrap();

        assert_matches!(
            mechanism,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(endpoint) if endpoint == url
        );
    }

    #[test]
    fn try_new_acf_header() {
        let mechanism =
            ClientAttestationChallengeMechanism::try_new_acf(None, &http_response(Some("the-nonce"))).unwrap();

        assert_matches!(
            mechanism,
            ClientAttestationChallengeMechanism::Header(nonce) if nonce == Nonce::from("the-nonce".to_string())
        );
    }

    #[test]
    fn try_new_acf_double_challenge_mechanism() {
        let url: Url = "https://issuer.example.com/challenge".parse().unwrap();

        let error =
            ClientAttestationChallengeMechanism::try_new_acf(Some(url), &http_response(Some("the-nonce"))).unwrap_err();

        assert_matches!(error, ClientAttestationError::DoubleChallengeMechanism);
    }

    #[test]
    fn try_new_acf_header_non_visible_ascii() {
        let mut builder = HttpResponse::builder();
        builder = builder.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, [0xffu8].as_slice());
        let response = Response::from(builder.body("").unwrap());

        let error = ClientAttestationChallengeMechanism::try_new_acf(None, &response).unwrap_err();

        assert_matches!(error, ClientAttestationError::ChallengeHeaderNonVisibleAscii);
    }

    #[test]
    fn new_pre_authorized_none() {
        let mechanism = ClientAttestationChallengeMechanism::new_pre_authorized(None);

        assert_matches!(mechanism, ClientAttestationChallengeMechanism::None);
    }

    #[test]
    fn new_pre_authorized_challenge_endpoint() {
        let url: Url = "https://issuer.example.com/challenge".parse().unwrap();

        let mechanism = ClientAttestationChallengeMechanism::new_pre_authorized(Some(url.clone()));

        assert_matches!(
            mechanism,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(endpoint) if endpoint == url
        );
    }

    /// Returns [`AuthorizationServerMetadata`] that fully supports Attestation-Based Client Authentication.
    fn oauth_metadata_with_client_attestation_support() -> AuthorizationServerMetadata {
        let mut metadata = AuthorizationServerMetadata::new(
            "https://issuer.example.com".parse().unwrap(),
            "https://issuer.example.com/token".parse().unwrap(),
        );
        metadata.token_endpoint_auth_methods_supported = Some([WIA_CLIENT_AUTH_METHOD.to_string()].into());
        metadata.client_attestation_signing_alg_values_supported = Some([JwsAlgorithm::ES256].into());
        metadata.client_attestation_pop_signing_alg_values_supported = Some([JwsAlgorithm::ES256].into());

        metadata
    }

    #[test]
    fn check_client_attestation_metadata_ok() {
        let metadata = oauth_metadata_with_client_attestation_support();

        check_client_attestation_metadata(&metadata).expect("client attestation metadata should be accepted");
    }

    #[test]
    fn check_client_attestation_metadata_no_token_endpoint_auth_methods() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.token_endpoint_auth_methods_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_wia_auth_method_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.token_endpoint_auth_methods_supported = Some(["client_secret_basic".to_string()].into());

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_signing_alg_values() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_signing_alg_values_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::ClientAttestationSigningAlgNotSupported(None))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_signing_alg_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::ClientAttestationSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_pop_signing_alg_values() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_pop_signing_alg_values_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::ClientAttestationPopSigningAlgNotSupported(None))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_pop_signing_alg_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_pop_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            check_client_attestation_metadata(&metadata),
            Err(ClientAttestationError::ClientAttestationPopSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }
}
