use std::ops::Add;

use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    mock::{generate_issuance_key_and_ca, SoftwareKeyFactory},
    server_keys::{KeyRing, SingleKeyRing},
    server_state::{MemorySessionStore, SessionState, SessionStore},
    Tdate,
};
use openid4vc::{
    credential::{CredentialRequests, CredentialResponses},
    dpop::Dpop,
    issuance_client::{HttpIssuerClient, IssuerClient, OpenidMessageClient},
    issuer::{AttributeService, Created, IssuanceData, Issuer},
    token::{AccessToken, TokenRequest, TokenRequestGrantType, TokenResponseWithPreviews},
    IssuerClientError,
};
use url::Url;

#[tokio::test]
async fn accept_issuance() {
    let sessions = MemorySessionStore::new();
    let (privkey, ca) = generate_issuance_key_and_ca().unwrap();
    let server_url = "https://example.com/".parse().unwrap();

    let issuer = Issuer::new(
        sessions,
        MockAttributeService,
        SingleKeyRing(privkey),
        &server_url,
        vec!["https://example.com".to_string()],
    );

    let message_client = MockOpenidMessageClient { issuer };

    let (session, _previews) = HttpIssuerClient::start_issuance(
        message_client,
        &server_url.join("issuance/").unwrap(),
        TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: "123".to_string().into(),
            },
            code_verifier: None,
            client_id: None,
            redirect_uri: None,
        },
    )
    .await
    .unwrap();

    let mdoc_copies = session
        .accept_issuance(
            &[(&ca).try_into().unwrap()],
            SoftwareKeyFactory::default(),
            &server_url.join("issuance/").unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(mdoc_copies.len(), 2);
    assert_eq!(mdoc_copies[0].cred_copies.len(), 2)
}

struct MockOpenidMessageClient<A, K, S> {
    issuer: Issuer<A, K, S>,
}

impl<A, K, S> OpenidMessageClient for MockOpenidMessageClient<A, K, S>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
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
            .unwrap();
        Ok((token_response, Some(dpop_nonce)))
    }

    async fn request_credentials(
        &self,
        _url: &url::Url,
        credential_requests: &CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponses, IssuerClientError> {
        let responses = self
            .issuer
            .process_batch_credential(
                AccessToken::from(access_token_header[5..].to_string()),
                Dpop::from(dpop_header.to_string()),
                credential_requests.clone(),
            )
            .await
            .unwrap();
        Ok(responses)
    }

    async fn reject(
        &self,
        _url: &url::Url,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<(), IssuerClientError> {
        self.issuer
            .process_reject_issuance(
                AccessToken::from(access_token_header[5..].to_string()),
                Dpop::from(dpop_header.to_string()),
                "batch_credential",
            )
            .await
            .unwrap();
        Ok(())
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
