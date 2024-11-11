use std::{num::NonZeroU8, ops::Add};

use assert_matches::assert_matches;
use chrono::{Days, Utc};
use ciborium::Value;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use rstest::rstest;
use url::Url;

use nl_wallet_mdoc::{
    server_keys::{test::SingleKeyRing, KeyPair},
    unsigned::{Entry, UnsignedMdoc},
    utils::{issuer_auth::IssuerRegistration, x509::Certificate},
    Tdate,
};
use openid4vc::{
    credential::{
        CredentialRequest, CredentialRequestProof, CredentialRequests, CredentialResponse, CredentialResponses,
    },
    dpop::Dpop,
    issuance_session::{
        mock_wte, HttpIssuanceSession, IssuanceSession, IssuanceSessionError, IssuedCredentialCopies, VcMessageClient,
    },
    issuer::{AttributeService, Created, IssuanceData, Issuer},
    metadata::IssuerMetadata,
    oidc,
    server_state::{MemorySessionStore, MemoryWteTracker, SessionState},
    token::{AccessToken, CredentialPreview, TokenRequest, TokenResponseWithPreviews},
    CredentialErrorCode,
};
use wallet_common::{
    jwt::{JsonJwt, Jwt},
    keys::{
        poa::{Poa, PoaPayload},
        software_key_factory::SoftwareKeyFactory,
    },
    nonempty::NonEmpty,
    urls::BaseUrl,
};

type MockIssuer = Issuer<MockAttributeService, SingleKeyRing, MemorySessionStore<IssuanceData>, MemoryWteTracker>;

fn setup_mdoc(attestation_count: usize, copy_count: u8) -> (MockIssuer, Certificate, BaseUrl, SigningKey) {
    let ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let issuance_keypair = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

    setup(
        MockAttributeService {
            previews: mock_mdoc_attributes(issuance_keypair.certificate(), attestation_count, copy_count),
        },
        ca,
        issuance_keypair,
    )
}

fn setup(
    attr_service: MockAttributeService,
    ca: KeyPair,
    issuance_keypair: KeyPair,
) -> (MockIssuer, Certificate, BaseUrl, SigningKey) {
    let server_url: BaseUrl = "https://example.com/".parse().unwrap();
    let wte_issuer_privkey = SigningKey::random(&mut OsRng);

    let issuer = MockIssuer::new(
        MemorySessionStore::default(),
        attr_service,
        SingleKeyRing(issuance_keypair),
        &server_url,
        vec!["https://wallet.edi.rijksoverheid.nl".to_string()],
        *wte_issuer_privkey.verifying_key(),
        MemoryWteTracker::new(),
    );

    (
        issuer,
        ca.into(),
        server_url.join_base_url("issuance/"),
        wte_issuer_privkey,
    )
}

#[rstest]
#[tokio::test]
async fn accept_issuance(#[values(1, 2)] attestation_count: usize, #[values(1, 2)] copy_count: u8) {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(attestation_count, copy_count);
    let message_client = MockOpenidMessageClient::new(issuer);

    let (session, previews) = HttpIssuanceSession::start_issuance(
        message_client,
        server_url.clone(),
        TokenRequest::new_mock(),
        &[(&ca).try_into().unwrap()],
    )
    .await
    .unwrap();

    let key_factory = SoftwareKeyFactory::default();
    let wte = mock_wte(&key_factory, &wte_issuer_privkey).await;

    let issued_creds = session
        .accept_issuance(&[(&ca).try_into().unwrap()], key_factory, Some(wte), server_url)
        .await
        .unwrap();

    assert_eq!(issued_creds.len(), attestation_count);
    assert_eq!(issued_creds.first().unwrap().len(), copy_count as usize);

    issued_creds
        .into_iter()
        .zip(previews)
        .for_each(|(copies, preview)| match copies {
            IssuedCredentialCopies::MsoMdoc(mdocs) => mdocs
                .first()
                .compare_unsigned(match &preview {
                    CredentialPreview::MsoMdoc { unsigned_mdoc, .. } => unsigned_mdoc,
                })
                .unwrap(),
        });
}

#[tokio::test]
async fn reject_issuance() {
    let (issuer, ca, server_url, _) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient::new(issuer);

    let (session, _previews) = HttpIssuanceSession::start_issuance(
        message_client,
        server_url,
        TokenRequest::new_mock(),
        &[(&ca).try_into().unwrap()],
    )
    .await
    .unwrap();

    session.reject_issuance().await.unwrap();
}

async fn start_and_accept_err(
    message_client: MockOpenidMessageClient,
    server_url: BaseUrl,
    ca: Certificate,
    wte_issuer_privkey: SigningKey,
) -> IssuanceSessionError {
    let (session, _previews) = HttpIssuanceSession::start_issuance(
        message_client,
        server_url.clone(),
        TokenRequest::new_mock(),
        &[(&ca).try_into().unwrap()],
    )
    .await
    .unwrap();

    let key_factory = SoftwareKeyFactory::default();
    let wte = mock_wte(&key_factory, &wte_issuer_privkey).await;

    session
        .accept_issuance(&[(&ca).try_into().unwrap()], key_factory, Some(wte), server_url)
        .await
        .unwrap_err()
}

#[tokio::test]
async fn wrong_access_token() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        wrong_access_token: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidToken)
    );
}

#[tokio::test]
async fn invalid_dpop() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        invalidate_dpop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
    );
}

#[tokio::test]
async fn invalid_pop() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        invalidate_pop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert!(matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
    ));
}

#[tokio::test]
async fn invalid_poa() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        invalidate_poa: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
    );
}

#[tokio::test]
async fn no_poa() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        strip_poa: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
    );
}

#[tokio::test]
async fn no_wte() {
    let (issuer, ca, server_url, wte_issuer_privkey) = setup_mdoc(1, 1);
    let message_client = MockOpenidMessageClient {
        strip_wte: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, ca, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
    );
}

// Helpers and mocks

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
    invalidate_poa: bool,
    strip_poa: bool,
    strip_wte: bool,
}

impl MockOpenidMessageClient {
    fn new(issuer: MockIssuer) -> Self {
        Self {
            issuer,
            wrong_access_token: false,
            invalidate_dpop: false,
            invalidate_pop: false,
            invalidate_poa: false,
            strip_poa: false,
            strip_wte: false,
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

    fn credential_request(&self, mut credential_request: CredentialRequest) -> CredentialRequest {
        if self.invalidate_pop {
            let invalidated_proof = match credential_request.proof.as_ref().unwrap() {
                CredentialRequestProof::Jwt { jwt } => CredentialRequestProof::Jwt {
                    jwt: invalidate_jwt(&jwt.0).into(),
                },
            };
            credential_request.proof = Some(invalidated_proof);
        }

        if self.invalidate_poa {
            credential_request.poa = Some(Self::invalidate_poa(credential_request.poa.unwrap()));
        }

        if self.strip_poa {
            credential_request.poa.take();
        }

        if self.strip_wte {
            credential_request.attestations.take();
        }

        credential_request
    }

    fn credential_requests(&self, mut credential_requests: CredentialRequests) -> CredentialRequests {
        if self.invalidate_pop {
            let invalidated_request = self.credential_request(credential_requests.credential_requests.first().clone());

            let mut requests = credential_requests.credential_requests.into_inner();
            requests[0] = invalidated_request;
            credential_requests.credential_requests = requests.try_into().unwrap();
        }

        if self.invalidate_poa {
            credential_requests.poa = Some(Self::invalidate_poa(credential_requests.poa.unwrap()));
        }

        if self.strip_poa {
            credential_requests.poa.take();
        }

        if self.strip_wte {
            credential_requests.attestations.take();
        }

        credential_requests
    }

    fn invalidate_poa(poa: Poa) -> Poa {
        let mut jwts: Vec<Jwt<PoaPayload>> = poa.into(); // a poa always involves at least two keys
        jwts.pop();
        let jwts: NonEmpty<_> = jwts.try_into().unwrap(); // jwts always has at least one left after the pop();
        let poa: JsonJwt<PoaPayload> = jwts.try_into().unwrap();

        poa
    }
}

/// Invalidate a JWT by modifying the last character of its signature
fn invalidate_jwt(jwt: &str) -> String {
    let new_char = if !jwt.ends_with('A') { 'A' } else { 'B' };
    jwt[..jwt.len() - 1].to_string() + &new_char.to_string()
}

impl VcMessageClient for MockOpenidMessageClient {
    async fn discover_metadata(&self, url: &BaseUrl) -> Result<IssuerMetadata, IssuanceSessionError> {
        Ok(IssuerMetadata::new_mock(url))
    }

    async fn discover_oauth_metadata(&self, url: &BaseUrl) -> Result<oidc::Config, IssuanceSessionError> {
        let metadata = oidc::Config::new_mock(url);
        Ok(metadata)
    }

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
            .map_err(|err| IssuanceSessionError::TokenRequest(err.into()))?;
        Ok((token_response, Some(dpop_nonce)))
    }

    async fn request_credential(
        &self,
        _url: &Url,
        credential_request: &CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, IssuanceSessionError> {
        self.issuer
            .process_credential(
                self.access_token(access_token_header),
                self.dpop(dpop_header),
                self.credential_request(credential_request.clone()),
            )
            .await
            .map_err(|err| IssuanceSessionError::CredentialRequest(err.into()))
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
            .map_err(|err| IssuanceSessionError::CredentialRequest(err.into()))
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
            .map_err(|err| IssuanceSessionError::CredentialRequest(err.into()))
    }
}

const MOCK_DOCTYPES: [&str; 2] = ["com.example.pid", "com.example.address"];
const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

fn mock_mdoc_attributes(issuer_cert: &Certificate, attestation_count: usize, copy_count: u8) -> Vec<CredentialPreview> {
    (0..attestation_count)
        .map(|i| CredentialPreview::MsoMdoc {
            unsigned_mdoc: UnsignedMdoc {
                doc_type: MOCK_DOCTYPES[i].to_string(),
                copy_count: NonZeroU8::new(copy_count).unwrap(),
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    MOCK_DOCTYPES[i].to_string(),
                    MOCK_ATTRS
                        .iter()
                        .map(|(key, val)| Entry {
                            name: key.to_string(),
                            value: Value::Text(val.to_string()),
                        })
                        .collect(),
                )])
                .try_into()
                .unwrap(),
            },
            issuer: issuer_cert.clone(),
        })
        .collect()
}

struct MockAttributeService {
    previews: Vec<CredentialPreview>,
}

impl AttributeService for MockAttributeService {
    type Error = std::convert::Infallible;

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<NonEmpty<Vec<CredentialPreview>>, Self::Error> {
        Ok(self.previews.clone().try_into().unwrap())
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}
