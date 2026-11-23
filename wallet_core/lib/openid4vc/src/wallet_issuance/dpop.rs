use crypto::p256_der::DerSigningKey;
use derive_more::Debug;
use http::HeaderMap;
use http::StatusCode;
use http::header::WWW_AUTHENTICATE;
use oauth::dpop::DPOP_NONCE_HEADER_NAME;
use oauth::dpop::Dpop;
use oauth::dpop::DpopNonce;
use oauth::token::AccessToken;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use p256::elliptic_curve::zeroize::Zeroizing;
use reqwest::Method;
use reqwest::Response;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde_with::SerializeAs;
use url::Url;

use super::WalletIssuanceError;

/// The wallet's DPoP key and the authorization server's nonce. Persist both across the authorization redirect
/// so the token request can use the key established at PAR, including after an app restart.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DpopState {
    #[debug(skip)]
    #[serde(
        serialize_with = "DerSigningKey::serialize_as",
        deserialize_with = "deserialize_signing_key"
    )]
    pub signing_key: SigningKey,
    pub nonce: Option<DpopNonce>,
}

impl Default for DpopState {
    fn default() -> Self {
        Self {
            signing_key: SigningKey::generate(),
            nonce: None,
        }
    }
}

fn deserialize_signing_key<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SigningKey, D::Error> {
    let bytes = Vec::<u8>::deserialize(deserializer)?;
    // Sessions saved before the switch to PKCS#8 stored just the 32-byte P-256 scalar.
    if bytes.len() == 32 {
        return SigningKey::from_slice(&Zeroizing::new(bytes)).map_err(serde::de::Error::custom);
    }
    DerSigningKey::try_from(bytes)
        .map(DerSigningKey::into_inner)
        .map_err(serde::de::Error::custom)
}

/// Send a credential request with its access-token-bound DPoP proof. On a nonce challenge, sign a fresh proof
/// for the same request and retry once. The returned nonce is the one used in the successful attempt.
pub(super) async fn request_with_dpop_retry<T, Fut>(
    signing_key: &SigningKey,
    url: &Url,
    access_token: Option<&AccessToken>,
    mut nonce: Option<DpopNonce>,
    mut request: impl FnMut(Dpop) -> Fut,
) -> Result<(T, Option<DpopNonce>), WalletIssuanceError>
where
    Fut: Future<Output = Result<T, WalletIssuanceError>>,
{
    let proof = Dpop::new(signing_key, url.clone(), &Method::POST, access_token, nonce.clone())?;
    let response = match request(proof).await {
        Err(WalletIssuanceError::UseDpopNonce(new_nonce)) => {
            nonce = Some(new_nonce);
            let proof = Dpop::new(signing_key, url.clone(), &Method::POST, access_token, nonce.clone())?;
            request(proof).await.map_err(|error| match error {
                WalletIssuanceError::UseDpopNonce(_) => WalletIssuanceError::DpopNonceRejected,
                other => other,
            })?
        }
        result => result?,
    };
    Ok((response, nonce))
}

pub(super) fn nonce_from_headers(headers: &HeaderMap) -> Result<Option<DpopNonce>, WalletIssuanceError> {
    let mut values = headers.get_all(DPOP_NONCE_HEADER_NAME).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(WalletIssuanceError::MultipleDpopNonceHeaders);
    }
    let nonce = value
        .to_str()
        .map_err(WalletIssuanceError::DpopNonceHeader)?
        .parse()
        .map_err(WalletIssuanceError::DpopNonce)?;
    Ok(Some(nonce))
}

/// A resource-server challenge is conveyed in WWW-Authenticate; its body need not contain JSON.
pub(super) fn resource_nonce_challenge(response: &Response) -> Result<Option<DpopNonce>, WalletIssuanceError> {
    if response.status() != StatusCode::UNAUTHORIZED
        || !response
            .headers()
            .get_all(WWW_AUTHENTICATE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .any(has_nonce_challenge)
    {
        return Ok(None);
    }
    nonce_from_headers(response.headers())
}

fn has_nonce_challenge(value: &str) -> bool {
    // Commas delimit both challenges and auth-params, except inside quoted strings.
    let mut quoted = false;
    let mut escaped = false;
    let parts = value.split(|character| {
        if escaped {
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            return true;
        }
        false
    });

    let mut dpop = false;
    let mut error = None;
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (first, rest) = part.split_once(char::is_whitespace).unwrap_or((part, ""));
        let parameters = if !first.contains('=') && !rest.trim_start().starts_with('=') {
            if dpop && error == Some(true) {
                return true;
            }
            dpop = first.eq_ignore_ascii_case("DPoP");
            error = None;
            rest
        } else {
            part
        };
        if dpop
            && let Some((name, value)) = parameters.split_once('=')
            && name.trim().eq_ignore_ascii_case("error")
        {
            // Duplicate error parameters are ambiguous; do not retry them.
            error = Some(error.is_none() && is_nonce_error(value.trim()));
        }
    }
    dpop && error == Some(true)
}

fn is_nonce_error(value: &str) -> bool {
    let Some(quoted) = value.strip_prefix('"').and_then(|value| value.strip_suffix('"')) else {
        return value == "use_dpop_nonce";
    };
    let mut characters = quoted.chars();
    for expected in "use_dpop_nonce".chars() {
        let actual = match characters.next() {
            Some('\\') => characters.next(),
            character => character,
        };
        if actual != Some(expected) {
            return false;
        }
    }
    characters.next().is_none()
}

#[cfg(test)]
pub(super) mod tests {
    use std::assert_matches;
    use std::future::ready;

    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use crypto::keys::PublicKey;
    use error_category::Category;
    use error_category::ErrorCategory;
    use http::HeaderValue;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpClient;
    use httpmock::HttpMockRequest;
    use httpmock::MockServer;
    use jwt::nonce::Nonce;
    use oauth::dpop::DPOP_HEADER_NAME;
    use oauth::errors::RemoteErrorCode;
    use p256::pkcs8::DecodePrivateKey;
    use rstest::rstest;
    use serde_json::Value;
    use serde_json::json;
    use wscd::payload::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;
    use wscd::payload::wia::WIA_POP_HEADER_NAME;
    use wscd::payload::wia::WiaPop;
    use wscd::wia::mock::MockWiaClient;

    use super::*;
    use crate::client_auth::ClientAttestationChallengeMechanism;
    use crate::errors::CredentialErrorCode;
    use crate::token::VciTokenRequest;
    use crate::wallet_issuance::authorization::AttestedRequest;
    use crate::wallet_issuance::issuance_session::CredentialRequestBody;
    use crate::wallet_issuance::issuance_session::HttpVcMessageClient;
    use crate::wallet_issuance::issuance_session::VcMessageClient;

    pub fn request_nonce_matches(request: &HttpMockRequest, expected: Option<&str>) -> bool {
        let headers = request.headers();
        let Some(proof) = headers.get(DPOP_HEADER_NAME).and_then(|value| value.to_str().ok()) else {
            return false;
        };
        let Some(payload) = proof
            .split('.')
            .nth(1)
            .and_then(|payload| BASE64_URL_SAFE_NO_PAD.decode(payload).ok())
            .and_then(|payload| serde_json::from_slice::<Value>(&payload).ok())
        else {
            return false;
        };
        payload.get("nonce").and_then(Value::as_str) == expected
    }

    pub fn wia_challenge_matches(request: &HttpMockRequest, expected: Option<&str>) -> bool {
        let headers = request.headers();
        let pop = headers[WIA_POP_HEADER_NAME]
            .to_str()
            .unwrap()
            .parse::<WiaPop>()
            .unwrap();
        let (_, claims) = pop.dangerous_parse_unverified().unwrap();
        claims.challenge.as_ref().map(Nonce::as_ref) == expected
    }

    fn jwt_id(proof: &Dpop) -> String {
        let payload = BASE64_URL_SAFE_NO_PAD
            .decode(proof.to_string().split('.').nth(1).unwrap())
            .unwrap();
        serde_json::from_slice::<Value>(&payload).unwrap()["jti"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[rstest]
    #[case(None)]
    #[case(Some("old_nonce"))]
    #[tokio::test]
    async fn retry_signs_a_fresh_proof_for_the_same_request(#[case] initial_nonce: Option<&str>) {
        let key = SigningKey::generate();
        let url = "https://issuer.example/credential".parse().unwrap();
        let access_token = AccessToken::from("token".to_string());
        let mut proofs = Vec::new();
        let ((), nonce) = request_with_dpop_retry(
            &key,
            &url,
            Some(&access_token),
            initial_nonce.map(|nonce| nonce.parse().unwrap()),
            |proof| {
                proofs.push(proof);
                ready(if proofs.len() == 1 {
                    Err(WalletIssuanceError::UseDpopNonce("new_nonce".parse().unwrap()))
                } else {
                    Ok(())
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(nonce.unwrap().as_ref(), "new_nonce");
        assert_eq!(proofs.len(), 2);
        assert_ne!(jwt_id(&proofs[0]), jwt_id(&proofs[1]));
        for (proof, expected) in proofs.into_iter().zip([initial_nonce, Some("new_nonce")]) {
            let nonce = proof
                .verify_expecting_key(
                    PublicKey::from(*key.verifying_key()),
                    &url,
                    &Method::POST,
                    Some(&access_token),
                )
                .unwrap();
            assert_eq!(nonce.as_ref().map(DpopNonce::as_ref), expected);
        }
    }

    #[rstest]
    #[case(true, 2)]
    #[case(false, 1)]
    #[tokio::test]
    async fn retries_only_nonce_challenges_once(#[case] challenge: bool, #[case] expected_requests: usize) {
        let key = SigningKey::generate();
        let url = "https://issuer.example/token".parse().unwrap();
        let mut requests = 0;
        let result: Result<((), _), _> = request_with_dpop_retry(&key, &url, None, None, |_| {
            requests += 1;
            ready(if challenge {
                Err(WalletIssuanceError::UseDpopNonce("nonce".parse().unwrap()))
            } else {
                Err(WalletIssuanceError::PreAuthorizedCodeExpired)
            })
        })
        .await;
        assert!(result.is_err());
        assert_eq!(requests, expected_requests);
    }

    #[rstest]
    #[case(r#"DPoP error="use_dpop_nonce""#, true)]
    #[case("dpop\terror = use_dpop_nonce", true)]
    #[case(r#"DPoP realm="issuer, \"example\"", error="use_dpop_nonce", algs="ES256""#, true)]
    #[case(r#"Bearer realm="other", DPoP error="use_dpop_nonce""#, true)]
    #[case(r#"DPoP error="use_dpop_nonce", Bearer realm="other""#, true)]
    #[case(r#"DPoP error="use\_dpop_nonce""#, true)]
    #[case(r#"DPoP realm="issuer", , error="use_dpop_nonce""#, true)]
    #[case(r#"Bearer error="use_dpop_nonce""#, false)]
    #[case(r#"DPoP realm="example", Bearer error="use_dpop_nonce""#, false)]
    #[case(r#"DPoP error="invalid_dpop_proof", error_description="use_dpop_nonce""#, false)]
    #[case(r#"DPoP realm="error=\"use_dpop_nonce\"""#, false)]
    #[case(r#"DPoP error="use_dpop_nonce", error="invalid_dpop_proof""#, false)]
    #[case(r#"DPoP error="use_dpop_nonce",, error="invalid_dpop_proof""#, false)]
    #[case(r#"DPoP error="use_dpop_nonce"suffix"#, false)]
    #[case(r#"DPoP error="USE_DPOP_NONCE""#, false)]
    fn parses_only_the_dpop_error_parameter(#[case] value: &str, #[case] expected: bool) {
        assert_eq!(has_nonce_challenge(value), expected);
    }

    #[test]
    fn nonce_headers_are_validated() {
        let mut headers = HeaderMap::new();
        assert_eq!(nonce_from_headers(&headers).unwrap(), None);
        headers.insert(DPOP_NONCE_HEADER_NAME, HeaderValue::from_static("nonce"));
        assert_eq!(nonce_from_headers(&headers).unwrap().unwrap().as_ref(), "nonce");
        headers.append(DPOP_NONCE_HEADER_NAME, HeaderValue::from_static("other"));
        assert_matches!(
            nonce_from_headers(&headers),
            Err(WalletIssuanceError::MultipleDpopNonceHeaders)
        );
        headers.insert(DPOP_NONCE_HEADER_NAME, HeaderValue::from_static("invalid nonce"));
        assert_matches!(nonce_from_headers(&headers), Err(WalletIssuanceError::DpopNonce(_)));
    }

    #[test]
    fn legacy_dpop_session_is_saved_with_the_same_signing_key_in_pkcs8() {
        let key = SigningKey::generate();
        let legacy_session = json!({"signing_key": key.to_bytes().as_slice(), "nonce": "par_nonce"});
        let state: DpopState = serde_json::from_value(legacy_session).unwrap();

        let saved = serde_json::to_value(&state).unwrap();
        let der = Vec::<u8>::deserialize(&saved["signing_key"]).unwrap();
        let persisted_key = SigningKey::from_pkcs8_der(&der).unwrap();
        assert_eq!(persisted_key.verifying_key(), key.verifying_key());

        let restored: DpopState = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.signing_key.verifying_key(), key.verifying_key());
        assert_eq!(restored.nonce.unwrap().as_ref(), "par_nonce");
    }

    #[rstest]
    #[case(Vec::<u8>::new())]
    #[case(vec![1_u8; 31])]
    #[case(vec![0_u8; 32])]
    #[case(vec![u8::MAX; 32])]
    #[case(vec![1_u8; 33])]
    fn invalid_persisted_signing_keys_are_rejected(#[case] key: Vec<u8>) {
        assert!(serde_json::from_value::<DpopState>(json!({"signing_key": key, "nonce": null})).is_err());
    }

    #[rstest]
    #[case::endpoint_only(None, None, "challenge_1", "challenge_2", 2)]
    #[case::header_on_retry(None, Some("retry_header"), "challenge_1", "retry_header", 1)]
    #[case::fallback_after_par_header(Some("par_header"), None, "par_header", "challenge_1", 1)]
    #[case::replace_par_header(Some("par_header"), Some("retry_header"), "par_header", "retry_header", 0)]
    #[tokio::test]
    async fn token_retry_uses_response_header_before_challenge_endpoint(
        #[case] par_header: Option<&'static str>,
        #[case] retry_header: Option<&'static str>,
        #[case] first_challenge: &'static str,
        #[case] retry_challenge: &'static str,
        #[case] endpoint_calls: usize,
    ) {
        use std::sync::atomic::AtomicUsize;
        use std::sync::atomic::Ordering;

        let server = MockServer::start_async().await;
        let challenge = server
            .mock_async(|when, then| {
                let sequence = AtomicUsize::new(1);
                when.method(httpmock::Method::POST).path("/challenge");
                then.respond_with(move |_| {
                    let challenge = format!("challenge_{}", sequence.fetch_add(1, Ordering::Relaxed));
                    http::Response::builder()
                        .body(json!({"attestation_challenge": challenge}).to_string())
                        .unwrap()
                        .into()
                });
            })
            .await;
        let first = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/token")
                    .is_true(move |request| {
                        request_nonce_matches(request, None) && wia_challenge_matches(request, Some(first_challenge))
                    });
                let then = if let Some(header) = retry_header {
                    then.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, header)
                } else {
                    then
                };
                then.status(400)
                    .header(DPOP_NONCE_HEADER_NAME, "retry_nonce")
                    .json_body(json!({"error":"use_dpop_nonce"}));
            })
            .await;
        let second = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/token")
                    .is_true(move |request| {
                        request_nonce_matches(request, Some("retry_nonce"))
                            && wia_challenge_matches(request, Some(retry_challenge))
                    });
                then.status(200)
                    .json_body(json!({"access_token":"token", "token_type":"DPoP"}));
            })
            .await;
        let client = HttpVcMessageClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let url = server.url("/token").parse().unwrap();
        let request = VciTokenRequest::new_pre_authorized("code".to_string().into());
        let issuer = server.base_url().parse().unwrap();
        let mechanism =
            ClientAttestationChallengeMechanism::ChallengeEndpoint(server.url("/challenge").parse().unwrap())
                .after_response(par_header.map(|value| Nonce::from(value.to_owned())));
        let (response, _) = tokio::spawn(async move {
            AttestedRequest {
                endpoint: &url,
                audience: &issuer,
                challenge: mechanism,
            }
            .send_with_dpop_retry(
                &mut DpopState::default(),
                &MockWiaClient::new(),
                |url| client.request_challenge(url),
                async |proof, wia| client.request_token(url.clone(), &request, &proof, &wia).await,
            )
            .await
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(response.oauth_response.access_token.as_ref(), "token");
        first.assert_calls_async(1).await;
        second.assert_calls_async(1).await;
        challenge.assert_calls_async(endpoint_calls).await;
    }

    #[rstest]
    #[case::repeated_challenge(400, "use_dpop_nonce", Some("retry_nonce"), 2)]
    #[case::missing_nonce(400, "use_dpop_nonce", None, 1)]
    #[case::invalid_nonce(400, "use_dpop_nonce", Some("invalid nonce"), 1)]
    #[case::wrong_error(400, "invalid_grant", Some("retry_nonce"), 1)]
    #[case::wrong_status(401, "use_dpop_nonce", Some("retry_nonce"), 1)]
    #[tokio::test]
    async fn token_http_retry_requires_a_valid_challenge(
        #[case] status: u16,
        #[case] error_code: &str,
        #[case] nonce: Option<&str>,
        #[case] expected_requests: usize,
    ) {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST).path("/token");
                let then = then.status(status);
                let then = match nonce {
                    Some(nonce) => then.header(DPOP_NONCE_HEADER_NAME, nonce),
                    None => then,
                };
                then.json_body(json!({"error": error_code}));
            })
            .await;
        let client = HttpVcMessageClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let url = server.url("/token").parse().unwrap();
        let request = VciTokenRequest::new_pre_authorized("code".to_string().into());
        let error = AttestedRequest {
            endpoint: &url,
            audience: &server.base_url().parse().unwrap(),
            challenge: ClientAttestationChallengeMechanism::None,
        }
        .send_with_dpop_retry(
            &mut DpopState::default(),
            &MockWiaClient::new(),
            |url| client.request_challenge(url),
            async |proof, wia| client.request_token(url.clone(), &request, &proof, &wia).await,
        )
        .await
        .unwrap_err();
        if expected_requests == 2 {
            assert_matches!(error, WalletIssuanceError::DpopNonceRejected);
        } else if nonce == Some("invalid nonce") {
            assert_matches!(error, WalletIssuanceError::DpopNonce(_));
        } else {
            assert_matches!(error, WalletIssuanceError::VciTokenRequest(_));
        }
        mock.assert_calls_async(expected_requests).await;
    }

    #[rstest]
    #[case::challenge(401, r#"DPoP error="use_dpop_nonce""#, Some("retry_nonce"), 2)]
    #[case::missing_nonce(401, r#"DPoP error="use_dpop_nonce""#, None, 1)]
    #[case::invalid_nonce(401, r#"DPoP error="use_dpop_nonce""#, Some("invalid nonce"), 1)]
    #[case::wrong_scheme(401, r#"Bearer error="use_dpop_nonce""#, Some("retry_nonce"), 1)]
    #[case::wrong_error(401, r#"DPoP error="invalid_dpop_proof""#, Some("retry_nonce"), 1)]
    #[case::wrong_status(400, r#"DPoP error="use_dpop_nonce""#, Some("retry_nonce"), 1)]
    #[tokio::test]
    async fn credential_http_challenge_requires_valid_headers_and_stops_after_one_retry(
        #[case] status: u16,
        #[case] authenticate: &str,
        #[case] nonce: Option<&str>,
        #[case] expected_requests: usize,
    ) {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/credential")
                    .header("Authorization", "DPoP token")
                    .body("encrypted request");
                let then = then.status(status).header(WWW_AUTHENTICATE.as_str(), authenticate);
                let then = match nonce {
                    Some(nonce) => then.header(DPOP_NONCE_HEADER_NAME, nonce),
                    None => then,
                };
                // No JSON body: RFC 9449 conveys the resource-server challenge through headers alone.
                then.body("");
            })
            .await;
        let client = HttpVcMessageClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let url = server.url("/credential").parse().unwrap();
        let key = SigningKey::generate();
        let token = AccessToken::from("token".to_string());
        let error = request_with_dpop_retry(&key, &url, Some(&token), None, async |proof| {
            client
                .request_credential(
                    url.clone(),
                    CredentialRequestBody::Encrypted("encrypted request".to_string()),
                    &proof,
                    &token,
                )
                .await
        })
        .await
        .unwrap_err();
        if expected_requests == 2 {
            assert_matches!(error, WalletIssuanceError::DpopNonceRejected);
            assert_eq!(error.category(), Category::Critical);
        }
        mock.assert_calls_async(expected_requests).await;
    }

    #[tokio::test]
    async fn credential_http_retry_preserves_body_and_propagates_other_errors() {
        let server = MockServer::start_async().await;
        let first = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/credential")
                    .body("encrypted request")
                    .is_true(|request| request_nonce_matches(request, None));
                then.status(401)
                    .header(WWW_AUTHENTICATE.as_str(), r#"DPoP error="use_dpop_nonce""#)
                    .header(DPOP_NONCE_HEADER_NAME, "retry_nonce");
            })
            .await;
        let second = server
            .mock_async(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/credential")
                    .body("encrypted request")
                    .is_true(|request| request_nonce_matches(request, Some("retry_nonce")));
                then.status(400).json_body(json!({"error":"invalid_proof"}));
            })
            .await;
        let client = HttpVcMessageClient::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let url = server.url("/credential").parse().unwrap();
        let token = AccessToken::from("token".to_string());
        let body = CredentialRequestBody::Encrypted("encrypted request".to_string());
        let error = request_with_dpop_retry(&SigningKey::generate(), &url, Some(&token), None, async |proof| {
            client
                .request_credential(url.clone(), body.clone(), &proof, &token)
                .await
        })
        .await
        .unwrap_err();
        assert_matches!(error, WalletIssuanceError::CredentialRequest(response)
            if response.error == RemoteErrorCode::Known(CredentialErrorCode::InvalidProof));
        first.assert_calls_async(1).await;
        second.assert_calls_async(1).await;
    }
}
