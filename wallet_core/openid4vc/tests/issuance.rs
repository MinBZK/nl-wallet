use std::ops::Add;

use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    server_keys::{KeyPair, SingleKeyRing},
    server_state::{MemorySessionStore, SessionState},
    software_key_factory::SoftwareKeyFactory,
    utils::{issuer_auth::IssuerRegistration, x509::Certificate},
    Tdate,
};
use openid4vc::{
    credential::{CredentialErrorCode, CredentialRequestProof, CredentialRequests, CredentialResponses},
    dpop::Dpop,
    issuance_session::{HttpIssuanceSession, IssuanceSession, IssuanceSessionError, OpenidMessageClient},
    issuer::{AttributeService, Created, IssuanceData, Issuer},
    token::{AccessToken, TokenRequest, TokenRequestGrantType, TokenResponseWithPreviews},
};

type MockIssuer = Issuer<MockAttributeService, SingleKeyRing, MemorySessionStore<IssuanceData>>;

fn setup() -> (MockIssuer, Certificate, Url) {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let privkey = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();
    let server_url = "https://example.com/".parse().unwrap();

    let issuer = MockIssuer::new(
        MemorySessionStore::new(),
        MockAttributeService,
        SingleKeyRing(privkey),
        &server_url,
        vec!["https://example.com".to_string()],
    );

    (issuer, ca.into(), server_url.join("issuance/").unwrap())
}

#[tokio::test]
async fn accept_issuance() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient::new(issuer);

    let (session, previews) = HttpIssuanceSession::start_issuance(message_client, server_url.clone(), token_request())
        .await
        .unwrap();

    let mdoc_copies = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), server_url)
        .await
        .unwrap();

    assert_eq!(mdoc_copies.len(), 2);
    assert_eq!(mdoc_copies[0].cred_copies.len(), 2);

    mdoc_copies.into_iter().zip(previews).for_each(|(copies, preview)| {
        copies
            .cred_copies
            .first()
            .unwrap()
            .compare_unsigned(preview.as_ref())
            .unwrap()
    });
}

#[tokio::test]
async fn reject_issuance() {
    let (issuer, _, server_url) = setup();
    let message_client = MockOpenidMessageClient::new(issuer);

    let (session, _previews) = HttpIssuanceSession::start_issuance(message_client, server_url, token_request())
        .await
        .unwrap();

    session.reject_issuance().await.unwrap();
}

#[tokio::test]
async fn wrong_access_token() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        wrong_access_token: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let (session, _previews) = HttpIssuanceSession::start_issuance(message_client, server_url.clone(), token_request())
        .await
        .unwrap();

    let result = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), server_url)
        .await
        .unwrap_err();

    assert!(matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidToken)
    ));
}

#[tokio::test]
async fn invalid_dpop() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        invalidate_dpop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let (session, _previews) = HttpIssuanceSession::start_issuance(message_client, server_url.clone(), token_request())
        .await
        .unwrap();

    let result = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), server_url)
        .await
        .unwrap_err();

    assert!(matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidRequest)
    ));
}

#[tokio::test]
async fn invalid_pop() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        invalidate_pop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let (session, _previews) = HttpIssuanceSession::start_issuance(message_client, server_url.clone(), token_request())
        .await
        .unwrap();

    let result = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), server_url)
        .await
        .unwrap_err();

    assert!(matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
    ));
}

// Helpers and mocks

fn token_request() -> TokenRequest {
    TokenRequest {
        grant_type: TokenRequestGrantType::PreAuthorizedCode {
            pre_authorized_code: "123".to_string().into(),
        },
        code_verifier: None,
        client_id: None,
        redirect_uri: None,
    }
}

/// An implementation of [`OpenidMessageClient`] that sends its messages to the contained issuer
/// directly by function invocation, optionally allowing the caller to mess with the input to trigger
/// certain error cases.
///
/// NOTE: The specific way in which each message (the Token Request/Response and Credential Request/Response)
/// is sent over HTTP in OpenID4VCI (e.g. in header or POST body, or JSON or URL encoded) is part of the standard.
/// Using this mock implementation of `OpenidMessageClient` means that that part of the standard is not used,
/// since it bypasses HTTP altogether. Therefore, using this struct to test the OpenID4VCI implementation means
/// that the transport part of this implementation of the protocol is not tested.
struct MockOpenidMessageClient {
    issuer: MockIssuer,

    wrong_access_token: bool,
    invalidate_dpop: bool,
    invalidate_pop: bool,
}

impl MockOpenidMessageClient {
    fn new(issuer: MockIssuer) -> Self {
        Self {
            issuer,
            wrong_access_token: false,
            invalidate_dpop: false,
            invalidate_pop: false,
        }
    }
}

impl MockOpenidMessageClient {
    fn access_token(&self, access_token_header: &str) -> AccessToken {
        if self.wrong_access_token {
            let code = &access_token_header[32 + 5..]; // Strip "DPoP "
            AccessToken::from("0".repeat(32) + code)
        } else {
            AccessToken::from(access_token_header[5..].to_string())
        }
    }

    fn dpop(&self, dpop_header: &str) -> Dpop {
        if self.invalidate_dpop {
            Dpop::from(invalidate_jwt(dpop_header))
        } else {
            Dpop::from(dpop_header.to_string())
        }
    }

    fn credential_requests(&self, mut credential_requests: CredentialRequests) -> CredentialRequests {
        if self.invalidate_pop {
            let invalidated_proof = match credential_requests
                .credential_requests
                .first()
                .unwrap()
                .proof
                .as_ref()
                .unwrap()
            {
                CredentialRequestProof::Jwt { jwt } => CredentialRequestProof::Jwt {
                    jwt: invalidate_jwt(&jwt.0).into(),
                },
            };
            credential_requests.credential_requests[0].proof = Some(invalidated_proof);
            credential_requests
        } else {
            credential_requests
        }
    }
}

/// Invalidate a JWT by modifying the last character of its signature
fn invalidate_jwt(jwt: &str) -> String {
    let new_char = if !jwt.ends_with('A') { 'A' } else { 'B' };
    jwt[..jwt.len() - 1].to_string() + &new_char.to_string()
}

impl OpenidMessageClient for MockOpenidMessageClient {
    async fn request_token(
        &self,
        _url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuanceSessionError> {
        let (token_response, dpop_nonce) = self
            .issuer
            .process_token_request(token_request.clone(), dpop_header.clone())
            .await
            .map_err(|err| IssuanceSessionError::TokenRequest(Box::new(err.into())))?;
        Ok((token_response, Some(dpop_nonce)))
    }

    async fn request_credentials(
        &self,
        _url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuanceSessionError> {
        self.issuer
            .process_batch_credential(
                self.access_token(access_token_header),
                self.dpop(dpop_header),
                self.credential_requests(credential_requests.clone()),
            )
            .await
            .map_err(|err| IssuanceSessionError::CredentialRequest(Box::new(err.into())))
    }

    async fn reject(
        &self,
        _url: &Url,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<(), IssuanceSessionError> {
        self.issuer
            .process_reject_issuance(
                self.access_token(access_token_header),
                self.dpop(dpop_header),
                "batch_credential",
            )
            .await
            .map_err(|err| IssuanceSessionError::CredentialRequest(Box::new(err.into())))
    }
}

const MOCK_PID_DOCTYPE: &str = "com.example.pid";
const MOCK_ADDRESS_DOCTYPE: &str = "com.example.address";
const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

struct MockAttributeService;

impl AttributeService for MockAttributeService {
    type Error = std::convert::Infallible;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error> {
        Ok(vec![
            UnsignedMdoc {
                doc_type: MOCK_PID_DOCTYPE.to_string(),
                copy_count: 2,
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    MOCK_PID_DOCTYPE.to_string(),
                    MOCK_ATTRS
                        .iter()
                        .map(|(key, val)| Entry {
                            name: key.to_string(),
                            value: Value::Text(val.to_string()),
                        })
                        .collect(),
                )]),
            },
            UnsignedMdoc {
                doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                copy_count: 2,
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    MOCK_ADDRESS_DOCTYPE.to_string(),
                    MOCK_ATTRS
                        .iter()
                        .map(|(key, val)| Entry {
                            name: key.to_string(),
                            value: Value::Text(val.to_string()),
                        })
                        .collect(),
                )]),
            },
        ])
    }
}
