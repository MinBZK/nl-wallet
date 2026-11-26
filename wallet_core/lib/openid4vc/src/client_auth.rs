use error_category::ErrorCategory;
use http::HeaderMap;
use http::header::ACCEPT;
use http_utils::reqwest::HttpClient;
use indexmap::IndexSet;
use itertools::Itertools;
use jwt::nonce::Nonce;
use oauth::jose::JwsAlgorithm;
use oauth::metadata::oauth_metadata::AuthorizationServerMetadata;
use oauth::metadata::oauth_metadata::ClientAttestationMetadataExtension;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use wscd::payload::wia::WIA_CLIENT_AUTH_METHOD;
use wscd::payload::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ClientAttestationMetadataError {
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
    client_attestation_metadata_extension: &ClientAttestationMetadataExtension,
) -> Result<(), ClientAttestationMetadataError> {
    if !oauth_metadata
        .token_endpoint_auth_methods_supported
        .as_ref()
        .is_some_and(|auth_methods| auth_methods.contains(WIA_CLIENT_AUTH_METHOD))
    {
        return Err(ClientAttestationMetadataError::NoAttestationBasedClientAuthSupport);
    }

    if !client_attestation_metadata_extension
        .client_attestation_signing_alg_values_supported
        .as_ref()
        .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
    {
        return Err(ClientAttestationMetadataError::ClientAttestationSigningAlgNotSupported(
            client_attestation_metadata_extension
                .client_attestation_signing_alg_values_supported
                .clone(),
        ));
    }

    if !client_attestation_metadata_extension
        .client_attestation_pop_signing_alg_values_supported
        .as_ref()
        .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
    {
        return Err(
            ClientAttestationMetadataError::ClientAttestationPopSigningAlgNotSupported(
                client_attestation_metadata_extension
                    .client_attestation_pop_signing_alg_values_supported
                    .clone(),
            ),
        );
    }

    Ok(())
}

/// Source of the signed `challenge` claim in a wallet instance attestation proof of possession (WIA PoP).
/// The wallet can use this source for a PAR or token request and persists it across the authorization redirect.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAttestationChallengeMechanism {
    ChallengeEndpoint(Url),
    Header(Nonce),
    None,
    /// Use this response header for the next PoP, then fetch from the endpoint unless another header arrives.
    HeaderWithEndpoint {
        header: Nonce,
        endpoint: Url,
    },
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(critical)]
pub enum ClientAttestationChallengeMechanismError {
    #[error("OAuth-Client-Attestation-Challenge contained non-visible-ASCII bytes")]
    ChallengeHeaderNonVisibleAscii,
    #[error("OAuth-Client-Attestation-Challenge is empty")]
    EmptyChallengeHeader,
    #[error("response contains multiple OAuth-Client-Attestation-Challenge headers")]
    MultipleChallengeHeaders,
}

impl ClientAttestationChallengeMechanism {
    /// Select the token request's challenge source from server metadata and the successful PAR response.
    pub fn try_new_acf(
        challenge_endpoint: Option<Url>,
        http_response: &Response,
    ) -> Result<Self, ClientAttestationChallengeMechanismError> {
        let mechanism = challenge_endpoint.map_or(Self::None, Self::ChallengeEndpoint);
        Ok(mechanism.after_response(attestation_challenge_from_headers(http_response.headers())?))
    }

    /// A response header supplies the challenge for the next WIA PoP, even when the server advertises an endpoint.
    /// After using that header, fetch from the endpoint again unless the server sends a replacement.
    /// With header-only delivery, the last value is the only one available for a retry.
    /// See [Attestation-Based Client Authentication §6](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-10#section-6).
    pub(crate) fn after_response(self, header: Option<Nonce>) -> Self {
        match (self, header) {
            (Self::ChallengeEndpoint(endpoint) | Self::HeaderWithEndpoint { endpoint, .. }, Some(header)) => {
                Self::HeaderWithEndpoint { header, endpoint }
            }
            (Self::HeaderWithEndpoint { endpoint, .. }, None) => Self::ChallengeEndpoint(endpoint),
            (_, Some(header)) => Self::Header(header),
            (mechanism, None) => mechanism,
        }
    }

    /// A pre-authorized token request has no preceding PAR response. Its first attempt can only obtain
    /// a WIA challenge from the endpoint in server metadata.
    pub fn new_pre_authorized(challenge_endpoint: Option<Url>) -> Self {
        match challenge_endpoint {
            None => ClientAttestationChallengeMechanism::None,
            Some(url) => ClientAttestationChallengeMechanism::ChallengeEndpoint(url),
        }
    }
}

pub(crate) fn attestation_challenge_from_headers(
    headers: &HeaderMap,
) -> Result<Option<Nonce>, ClientAttestationChallengeMechanismError> {
    let mut values = headers.get_all(WIA_CLIENT_CHALLENGE_HEADER_NAME).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(ClientAttestationChallengeMechanismError::MultipleChallengeHeaders);
    }
    let challenge = value
        .to_str()
        .map_err(|_| ClientAttestationChallengeMechanismError::ChallengeHeaderNonVisibleAscii)?;
    if challenge.trim().is_empty() {
        return Err(ClientAttestationChallengeMechanismError::EmptyChallengeHeader);
    }
    Ok(Some(Nonce::from(challenge.to_owned())))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttestationChallenge {
    pub attestation_challenge: Nonce,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ClientAttestationChallengeError {
    #[error("could not request WIA challenge: {0}")]
    #[category(pd)]
    ChallengeRequest(#[source] reqwest::Error),

    #[error("WIA challenge endpoint returned an error status: {0}")]
    #[category(pd)]
    ChallengeStatus(#[source] reqwest::Error),

    #[error("could not parse WIA challenge response: {0}")]
    #[category(pd)]
    ChallengeBody(#[source] reqwest::Error),
}

pub async fn fetch_client_auth_challenge(
    http_client: &HttpClient,
    challenge_endpoint: Url,
) -> Result<Nonce, ClientAttestationChallengeError> {
    let challenge = http_client
        .post(challenge_endpoint, |builder| {
            builder.header(ACCEPT, mime::APPLICATION_JSON.as_ref())
        })
        .await
        .map_err(ClientAttestationChallengeError::ChallengeRequest)?
        .error_for_status()
        .map_err(ClientAttestationChallengeError::ChallengeStatus)?
        .json::<AttestationChallenge>()
        .await
        .map_err(ClientAttestationChallengeError::ChallengeBody)?
        .attestation_challenge;

    Ok(challenge)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use http::Response as HttpResponse;
    use jwt::nonce::Nonce;
    use reqwest::Response;
    use url::Url;
    use wscd::payload::wia::WIA_CLIENT_AUTH_METHOD;
    use wscd::payload::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;

    use super::AuthorizationServerMetadata;
    use super::ClientAttestationChallengeMechanism;
    use super::ClientAttestationChallengeMechanismError;
    use super::ClientAttestationMetadataError;
    use super::ClientAttestationMetadataExtension;
    use super::JwsAlgorithm;
    use super::check_client_attestation_metadata;

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
    fn try_new_acf_preserves_header_and_endpoint_when_serialized() {
        let url: Url = "https://issuer.example.com/challenge".parse().unwrap();
        let mechanism =
            ClientAttestationChallengeMechanism::try_new_acf(Some(url.clone()), &http_response(Some("par-challenge")))
                .unwrap();
        let restored = serde_json::from_value(serde_json::to_value(mechanism).unwrap()).unwrap();
        assert_matches!(
            restored,
            ClientAttestationChallengeMechanism::HeaderWithEndpoint { header, endpoint }
                if header.as_ref() == "par-challenge" && endpoint == url
        );
    }

    #[test]
    fn try_new_acf_header_non_visible_ascii() {
        let mut builder = HttpResponse::builder();
        builder = builder.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, [0xffu8].as_slice());
        let response = Response::from(builder.body("").unwrap());

        let error = ClientAttestationChallengeMechanism::try_new_acf(None, &response).unwrap_err();

        assert_matches!(
            error,
            ClientAttestationChallengeMechanismError::ChallengeHeaderNonVisibleAscii
        );
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

    /// Returns metadata that fully supports Attestation-Based Client Authentication.
    fn oauth_metadata_with_client_attestation_support()
    -> (AuthorizationServerMetadata, ClientAttestationMetadataExtension) {
        let mut oauth_metadata = AuthorizationServerMetadata::new(
            "https://issuer.example.com".parse().unwrap(),
            "https://issuer.example.com/token".parse().unwrap(),
        );
        oauth_metadata.token_endpoint_auth_methods_supported = Some([WIA_CLIENT_AUTH_METHOD.to_string()].into());

        let client_attestation_metadata_extension = ClientAttestationMetadataExtension {
            client_attestation_signing_alg_values_supported: Some([JwsAlgorithm::ES256].into()),
            client_attestation_pop_signing_alg_values_supported: Some([JwsAlgorithm::ES256].into()),
            ..Default::default()
        };

        (oauth_metadata, client_attestation_metadata_extension)
    }

    #[test]
    fn check_client_attestation_metadata_ok() {
        let (oauth_metadata, client_attestation_metadata_extension) = oauth_metadata_with_client_attestation_support();

        check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension)
            .expect("client attestation metadata should be accepted");
    }

    #[test]
    fn check_client_attestation_metadata_no_token_endpoint_auth_methods() {
        let (mut oauth_metadata, client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        oauth_metadata.token_endpoint_auth_methods_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_wia_auth_method_not_supported() {
        let (mut oauth_metadata, client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        oauth_metadata.token_endpoint_auth_methods_supported = Some(["client_secret_basic".to_string()].into());

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_signing_alg_values() {
        let (oauth_metadata, mut client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        client_attestation_metadata_extension.client_attestation_signing_alg_values_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::ClientAttestationSigningAlgNotSupported(
                None
            ))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_signing_alg_not_supported() {
        let (oauth_metadata, mut client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        client_attestation_metadata_extension.client_attestation_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::ClientAttestationSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_pop_signing_alg_values() {
        let (oauth_metadata, mut client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        client_attestation_metadata_extension.client_attestation_pop_signing_alg_values_supported = None;

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::ClientAttestationPopSigningAlgNotSupported(None))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_pop_signing_alg_not_supported() {
        let (oauth_metadata, mut client_attestation_metadata_extension) =
            oauth_metadata_with_client_attestation_support();
        client_attestation_metadata_extension.client_attestation_pop_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            check_client_attestation_metadata(&oauth_metadata, &client_attestation_metadata_extension),
            Err(ClientAttestationMetadataError::ClientAttestationPopSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }
}
