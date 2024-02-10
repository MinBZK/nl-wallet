use std::ops::Add;

use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    mock::{generate_issuance_key_and_ca, SoftwareKeyFactory},
    server_keys::SingleKeyRing,
    server_state::{MemorySessionStore, SessionState},
    utils::x509::Certificate,
    Tdate,
};
use openid4vc::{
    credential::{CredentialErrorType, CredentialRequests, CredentialResponses},
    dpop::Dpop,
    issuance_client::{HttpIssuerClient, IssuerClient, OpenidMessageClient},
    issuer::{AttributeService, Created, IssuanceData, Issuer},
    token::{AccessToken, TokenRequest, TokenRequestGrantType, TokenResponseWithPreviews},
    IssuerClientError,
};

type MockIssuer = Issuer<MockAttributeService, SingleKeyRing, MemorySessionStore<IssuanceData>>;

fn setup() -> (MockIssuer, Certificate, Url) {
    let (privkey, ca) = generate_issuance_key_and_ca().unwrap();
    let server_url = "https://example.com/".parse().unwrap();

    let issuer = MockIssuer::new(
        MemorySessionStore::new(),
        MockAttributeService,
        SingleKeyRing(privkey),
        &server_url,
        vec!["https://example.com".to_string()],
    );

    (issuer, ca, server_url.join("issuance/").unwrap())
}

#[tokio::test]
async fn accept_issuance() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        issuer,
        wrong_access_token: false,
    };

    let (session, _previews) = HttpIssuerClient::start_issuance(message_client, &server_url, token_request())
        .await
        .unwrap();

    let mdoc_copies = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), &server_url)
        .await
        .unwrap();

    assert_eq!(mdoc_copies.len(), 2);
    assert_eq!(mdoc_copies[0].cred_copies.len(), 2)
}

#[tokio::test]
async fn reject_issuance() {
    let (issuer, _, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        issuer,
        wrong_access_token: false,
    };

    let (session, _previews) = HttpIssuerClient::start_issuance(message_client, &server_url, token_request())
        .await
        .unwrap();

    session.reject_issuance().await.unwrap();
}

#[tokio::test]
async fn wrong_access_token() {
    let (issuer, ca, server_url) = setup();
    let message_client = MockOpenidMessageClient {
        issuer,
        wrong_access_token: true,
    };

    let (session, _previews) = HttpIssuerClient::start_issuance(message_client, &server_url, token_request())
        .await
        .unwrap();

    let result = session
        .accept_issuance(&[(&ca).try_into().unwrap()], SoftwareKeyFactory::default(), &server_url)
        .await
        .unwrap_err();

    assert!(matches!(
        result,
        IssuerClientError::CredentialRequest(err) if matches!(err.error, CredentialErrorType::InvalidToken)
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
/// directly by function invocation.
struct MockOpenidMessageClient {
    issuer: MockIssuer,

    wrong_access_token: bool,
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
}

impl OpenidMessageClient for MockOpenidMessageClient {
    async fn request_token(
        &self,
        _url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
    ) -> Result<(TokenResponseWithPreviews, Option<String>), IssuerClientError> {
        let (token_response, dpop_nonce) = self
            .issuer
            .process_token_request(token_request.clone(), dpop_header.clone())
            .await
            .map_err(|err| IssuerClientError::TokenRequest(Box::new(err.into())))?;
        Ok((token_response, Some(dpop_nonce)))
    }

    async fn request_credentials(
        &self,
        _url: &Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuerClientError> {
        self.issuer
            .process_batch_credential(
                self.access_token(access_token_header),
                Dpop::from(dpop_header.to_string()),
                credential_requests.clone(),
            )
            .await
            .map_err(|err| IssuerClientError::CredentialRequest(Box::new(err.into())))
    }

    async fn reject(&self, _url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), IssuerClientError> {
        self.issuer
            .process_reject_issuance(
                self.access_token(access_token_header),
                Dpop::from(dpop_header.to_string()),
                "batch_credential",
            )
            .await
            .map_err(|err| IssuerClientError::CredentialRequest(Box::new(err.into())))
    }
}

const MOCK_PID_DOCTYPE: &str = "com.example.pid";
const MOCK_ADDRESS_DOCTYPE: &str = "com.example.address";
const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

struct MockAttributeService;

impl AttributeService for MockAttributeService {
    type Error = openid4vc::IssuerClientError; // arbitrary type that implements the required bounds

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
