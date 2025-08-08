use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use assert_matches::assert_matches;
use chrono::Days;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use rstest::rstest;
use rustls_pki_types::TrustAnchor;
use url::Url;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::IntoCredentialPayload;
use attestation_data::issuable_document::IssuableDocument;
use attestation_data::x509::generate::mock::generate_issuer_mock;
use attestation_types::claim_path::ClaimPath;
use attestation_types::qualification::AttestationQualification;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use http_utils::urls::BaseUrl;
use jwt::JsonJwt;
use jwt::Jwt;
use openid4vc::CredentialErrorCode;
use openid4vc::Format;
use openid4vc::credential::CredentialRequest;
use openid4vc::credential::CredentialRequestProof;
use openid4vc::credential::CredentialRequests;
use openid4vc::credential::CredentialResponse;
use openid4vc::credential::CredentialResponses;
use openid4vc::dpop::Dpop;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::issuance_session::VcMessageClient;
use openid4vc::issuer::AttestationTypeConfig;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::WteConfig;
use openid4vc::metadata::IssuerMetadata;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::oidc;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::MemoryWteTracker;
use openid4vc::token::AccessToken;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenResponseWithPreviews;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::ClaimMetadata;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;
use wscd::PoaPayload;
use wscd::mock_remote::MockRemoteKeyFactory;

type MockIssuer = Issuer<MockAttributeService, SigningKey, MemorySessionStore<IssuanceData>, MemoryWteTracker>;

fn setup_mock_issuer(attestation_count: NonZeroUsize) -> (MockIssuer, TrustAnchor<'static>, BaseUrl, SigningKey) {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_keypair = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();

    setup(
        MockAttributeService {
            attestations: mock_issuable_attestation(attestation_count),
        },
        &ca,
        &issuance_keypair,
    )
}

fn setup(
    attr_service: MockAttributeService,
    ca: &Ca,
    issuance_keypair: &KeyPair,
) -> (MockIssuer, TrustAnchor<'static>, BaseUrl, SigningKey) {
    let server_url: BaseUrl = "https://example.com/".parse().unwrap();
    let wte_issuer_privkey = SigningKey::random(&mut OsRng);
    let trust_anchor = ca.to_trust_anchor().to_owned();

    let attestation_config = MOCK_ATTESTATION_TYPES
        .iter()
        .map(|attestation_type| {
            let (_, _, metadata_documents) =
                TypeMetadataDocuments::from_single_example(mock_type_metadata(attestation_type));

            (
                attestation_type.to_string(),
                AttestationTypeConfig::try_new(
                    attestation_type,
                    // KeyPair doesn't implement clone, so manually construct a new KeyPair.
                    KeyPair::new_from_signing_key(
                        issuance_keypair.private_key().clone(),
                        issuance_keypair.certificate().clone(),
                    )
                    .unwrap(),
                    Days::new(365),
                    IndexMap::from([(Format::MsoMdoc, 4.try_into().unwrap())]),
                    issuance_keypair
                        .certificate()
                        .san_dns_name_or_uris()
                        .unwrap()
                        .into_first(),
                    AttestationQualification::default(),
                    metadata_documents,
                )
                .unwrap(),
            )
        })
        .collect::<HashMap<_, _>>()
        .into();

    let issuer = MockIssuer::new(
        Arc::new(MemorySessionStore::default()),
        attr_service,
        attestation_config,
        &server_url,
        vec![MOCK_WALLET_CLIENT_ID.to_string()],
        Some(WteConfig {
            wte_issuer_pubkey: wte_issuer_privkey.verifying_key().into(),
            wte_tracker: Arc::new(MemoryWteTracker::new()),
        }),
    );

    (
        issuer,
        trust_anchor,
        server_url.join_base_url("issuance/"),
        wte_issuer_privkey,
    )
}

#[rstest]
#[tokio::test]
async fn accept_issuance(
    #[values(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let (issuer, trust_anchor, server_url, wua_signing_key) = setup_mock_issuer(attestation_count);
    let trust_anchors = &[trust_anchor];
    let message_client = MockOpenidMessageClient::new(issuer);
    let copy_count = 4;

    let session = HttpIssuanceSession::start_issuance(
        message_client,
        server_url.clone(),
        TokenRequest::new_mock(),
        trust_anchors,
    )
    .await
    .unwrap();

    let key_factory = MockRemoteKeyFactory::new_with_wua_signing_key(wua_signing_key);

    let issued_creds = session
        .accept_issuance(trust_anchors, &key_factory, true)
        .await
        .unwrap();

    assert_eq!(issued_creds.len(), attestation_count.get());
    assert_eq!(issued_creds.first().unwrap().copies.as_ref().len().get(), copy_count);

    issued_creds
        .into_iter()
        .zip(session.normalized_credential_preview().iter())
        .for_each(|(credential, preview_data)| {
            credential
                .copies
                .into_inner()
                .into_iter()
                .for_each(|issued_credential| match issued_credential {
                    IssuedCredential::MsoMdoc(mdoc) => {
                        let payload = (*mdoc)
                            .into_credential_payload(&preview_data.normalized_metadata)
                            .unwrap();
                        assert_eq!(payload.previewable_payload, preview_data.content.credential_payload);
                    }
                    IssuedCredential::SdJwt(_) => {
                        panic!("SdJwt should not be issued");
                    }
                })
        });
}

#[tokio::test]
async fn reject_issuance() {
    let (issuer, trust_anchor, server_url, _) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient::new(issuer);

    let session =
        HttpIssuanceSession::start_issuance(message_client, server_url, TokenRequest::new_mock(), &[trust_anchor])
            .await
            .unwrap();

    session.reject_issuance().await.unwrap();
}

async fn start_and_accept_err(
    message_client: MockOpenidMessageClient,
    server_url: BaseUrl,
    trust_anchor: TrustAnchor<'static>,
    wua_issuer_privkey: SigningKey,
) -> IssuanceSessionError {
    let trust_anchors = &[trust_anchor];
    let session = HttpIssuanceSession::start_issuance(
        message_client,
        server_url.clone(),
        TokenRequest::new_mock(),
        trust_anchors,
    )
    .await
    .unwrap();

    let key_factory = MockRemoteKeyFactory::new_with_wua_signing_key(wua_issuer_privkey);

    session
        .accept_issuance(trust_anchors, &key_factory, true)
        .await
        .unwrap_err()
}

#[tokio::test]
async fn wrong_access_token() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        wrong_access_token: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidToken)
    );
}

#[tokio::test]
async fn invalid_dpop() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        invalidate_dpop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
    );
}

#[tokio::test]
async fn invalid_pop() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        invalidate_pop: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
    assert!(matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
    ));
}

#[tokio::test]
async fn invalid_poa() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        invalidate_poa: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
    );
}

#[tokio::test]
async fn no_poa() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        strip_poa: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
    assert_matches!(
        result,
        IssuanceSessionError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
    );
}

#[tokio::test]
async fn no_wte() {
    let (issuer, trust_anchor, server_url, wte_issuer_privkey) = setup_mock_issuer(NonZeroUsize::new(1).unwrap());
    let message_client = MockOpenidMessageClient {
        strip_wte: true,
        ..MockOpenidMessageClient::new(issuer)
    };

    let result = start_and_accept_err(message_client, server_url, trust_anchor, wte_issuer_privkey).await;
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
        let jwts: VecNonEmpty<_> = jwts.try_into().unwrap(); // jwts always has at least one left after the pop();
        let poa: JsonJwt<PoaPayload> = jwts.try_into().unwrap();

        poa.into()
    }
}

/// Invalidate a JWT by modifying the last character of its signature
fn invalidate_jwt(jwt: &str) -> String {
    let new_char = if !jwt.ends_with('A') { 'A' } else { 'B' };
    jwt[..jwt.len() - 1].to_string() + &new_char.to_string()
}

impl VcMessageClient for MockOpenidMessageClient {
    fn client_id(&self) -> &str {
        MOCK_WALLET_CLIENT_ID
    }

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

const MOCK_ATTESTATION_TYPES: [&str; 2] = ["com.example.pid", "com.example.address"];
const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

fn mock_type_metadata(vct: &str) -> TypeMetadata {
    TypeMetadata::try_new(UncheckedTypeMetadata {
        vct: vct.to_string(),
        claims: MOCK_ATTRS
            .iter()
            .map(|(key, _)| ClaimMetadata {
                path: vec![ClaimPath::SelectByKey(key.to_string())].try_into().unwrap(),
                display: vec![ClaimDisplayMetadata {
                    lang: "en".to_string(),
                    label: key.to_string(),
                    description: None,
                }],
                sd: ClaimSelectiveDisclosureMetadata::Allowed,
                svg_id: None,
            })
            .collect(),
        ..UncheckedTypeMetadata::empty_example()
    })
    .unwrap()
}

fn mock_issuable_attestation(attestation_count: NonZeroUsize) -> VecNonEmpty<IssuableDocument> {
    (0..attestation_count.get())
        .map(|i| {
            IssuableDocument::try_new(
                MOCK_ATTESTATION_TYPES[i].to_string(),
                IndexMap::from_iter(MOCK_ATTRS.iter().map(|(key, val)| {
                    (
                        key.to_string(),
                        Attribute::Single(AttributeValue::Text(val.to_string())),
                    )
                }))
                .into(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

struct MockAttributeService {
    attestations: VecNonEmpty<IssuableDocument>,
}

impl AttributeService for MockAttributeService {
    type Error = std::convert::Infallible;

    async fn attributes(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        Ok(self.attestations.clone())
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}
