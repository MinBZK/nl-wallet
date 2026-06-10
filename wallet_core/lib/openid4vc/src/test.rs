use std::num::NonZeroU8;
use std::num::NonZeroUsize;
use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use attestation_types::qualification::AttestationQualification;
use chrono::DateTime;
use chrono::Days;
use chrono::Utc;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::ClaimMetadata;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use token_status_list::status_list_service::mock::MockStatusListService;
use token_status_list::status_list_service::mock::generate_status_claims;
use url::Url;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::authorization::PkceCodeChallenge;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::authorizing_issuer::AuthorizingIssuer;
use crate::credential_configurations::CredentialConfigurationParameters;
use crate::issuable_document::IssuableDocument;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::issuer::WiaConfig;
use crate::issuer_identifier::IssuerIdentifier;
use crate::mock::MOCK_WALLET_CLIENT_ID;
use crate::nonce::memory_store::MemoryNonceStore;
use crate::par::PAR_TTL;
use crate::pkce::PKCE_FLOW_TTL;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::server_state::MemorySessionStore;
use crate::store::MemoryStore;
use crate::store::Store;
use crate::token::TokenRequest;

pub const MOCK_ATTESTATION_TYPES: [&str; 2] = ["com.example.pid", "com.example.address"];
pub const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];
pub const MOCK_UPSTREAM_CLIENT_ID: &str = "mock_upstream_client_id";

pub type MockIssuer<G = TimeGenerator> =
    Issuer<SigningKey, MockStatusListService, MemorySessionStore<IssuanceData, G>, MemoryNonceStore>;

pub type MockAuthorizingIssuer<G = TimeGenerator> = AuthorizingIssuer<
    SigningKey,
    MockStatusListService,
    MemorySessionStore<IssuanceData, G>,
    MemoryNonceStore,
    MemoryStore<String, VciAuthorizationRequest>,
    StaticAuthorizationCodeFlow,
>;

pub fn mock_type_metadata(vct: &str) -> TypeMetadata {
    TypeMetadata::try_new(UncheckedTypeMetadata {
        vct: vct.to_string(),
        claims: MOCK_ATTRS
            .iter()
            .map(|(key, _)| ClaimMetadata {
                path: vec_nonempty![ClaimPath::SelectByKey(key.to_string())],
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

pub fn mock_issuable_documents(document_count: NonZeroUsize) -> VecNonEmpty<IssuableDocument> {
    (0..document_count.get())
        .map(|i| {
            IssuableDocument::try_new_with_random_id(
                Format::SdJwt,
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

/// Test-only implementation of [`AuthorizationCodeFlow`] driving the auth-code path: `authorize` performs the same
/// wallet ↔ upstream PKCE bridge bookkeeping a real flow would (so wallet PKCE rejection is still
/// exercised at `/token`), rewrites the upstream client_id, and returns a query-encoded redirect
/// to the configured upstream URL. `issuables` consumes the bridge and returns preloaded docs.
pub struct StaticAuthorizationCodeFlow {
    upstream_url: Url,
    bridge: Arc<MemoryStore<String, String>>,
    documents: VecNonEmpty<IssuableDocument>,
}

impl StaticAuthorizationCodeFlow {
    pub fn new(upstream_url: Url, documents: VecNonEmpty<IssuableDocument>) -> Self {
        Self {
            upstream_url,
            bridge: Arc::new(MemoryStore::new(PKCE_FLOW_TTL)),
            documents,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StaticAuthorizationCodeFlowError {
    #[error("only S256 code_challenge_method is supported")]
    UnsupportedCodeChallenge,

    #[error("PKCE bridge store error: {0}")]
    BridgeStore(#[source] std::convert::Infallible),

    #[error("encoding upstream authorization request as query string failed: {0}")]
    Encode(#[source] serde_qs::Error),

    #[error("missing wallet code_verifier on token request")]
    MissingCodeVerifier,

    #[error("wallet code_verifier does not match any stored PKCE bridge entry")]
    PkceVerificationFailed,
}

// TODO (PVW-5953): too much logic in this stub implementation
impl AuthorizationCodeFlow for StaticAuthorizationCodeFlow {
    type Error = StaticAuthorizationCodeFlowError;

    async fn authorize(&self, mut request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
        let wallet_code_challenge = match &request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge.clone(),
            PkceCodeChallenge::Plain { .. } => {
                return Err(StaticAuthorizationCodeFlowError::UnsupportedCodeChallenge);
            }
        };
        // Generate an upstream verifier; substitute the upstream challenge into the request; and
        // store the upstream verifier keyed by the wallet challenge so `issuables` can verify it.
        let upstream_pkce = S256PkcePair::generate();
        request.code_challenge = PkceCodeChallenge::S256 {
            code_challenge: upstream_pkce.code_challenge().to_string(),
        };
        self.bridge
            .store(wallet_code_challenge, upstream_pkce.into_code_verifier())
            .await
            .map_err(StaticAuthorizationCodeFlowError::BridgeStore)?;

        request.oauth_request.client_id = MOCK_UPSTREAM_CLIENT_ID.to_string();

        let query_string = serde_qs::to_string(&request).map_err(StaticAuthorizationCodeFlowError::Encode)?;
        let mut redirect_url = self.upstream_url.clone();
        redirect_url.set_query(Some(&query_string));

        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }

    async fn issuables(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let wallet_code_verifier = token_request
            .code_verifier
            .as_deref()
            .ok_or(StaticAuthorizationCodeFlowError::MissingCodeVerifier)?;
        let wallet_code_challenge = S256PkcePair::challenge_for(wallet_code_verifier);

        self.bridge
            .consume(&wallet_code_challenge)
            .await
            .map_err(StaticAuthorizationCodeFlowError::BridgeStore)?
            .ok_or(StaticAuthorizationCodeFlowError::PkceVerificationFailed)?;

        Ok(self.documents.clone())
    }
}

pub fn setup_mock_issuer<G>(
    issuer_identifier: IssuerIdentifier,
    attestation_count: NonZeroUsize,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
) -> (MockIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_keypair = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
    let trust_anchors = TrustAnchors::from(&ca);
    let wia_keypair = ca.generate_wia_mock().unwrap();

    let config_params = MOCK_ATTESTATION_TYPES[..attestation_count.get()]
        .iter()
        .copied()
        .map(|attestation_type| {
            let mut status_list = MockStatusListService::new();
            status_list.expect_obtain_status_claims().returning(|_, _, copies| {
                let uri = format!("https://tsl.example.com/{}", attestation_type.replace(':', "-"))
                    .parse()
                    .unwrap();
                Ok(generate_status_claims(&uri, copies))
            });
            status_list
                .expect_start_refresh_job()
                .return_once(|| tokio::task::spawn(async {}).abort_handle());

            let (_, _, metadata_documents) =
                TypeMetadataDocuments::from_single_example(mock_type_metadata(attestation_type));

            let params = CredentialConfigurationParameters {
                format: Format::SdJwt,
                attestation_type: attestation_type.to_string(),
                key_pair: KeyPair::new_from_signing_key(
                    issuance_keypair.private_key().clone(),
                    issuance_keypair.certificate().clone(),
                )
                .unwrap(),
                valid_days: Days::new(365),
                status_list,
                issuer_uri: issuance_keypair
                    .certificate()
                    .san_dns_name_or_uris()
                    .unwrap()
                    .into_first(),
                attestation_qualification: AttestationQualification::default(),
                metadata_documents,
            };

            (attestation_type.to_string().into(), params)
        })
        .collect();

    let issuer = MockIssuer::try_new(
        issuer_identifier,
        NonZeroU8::new(4).unwrap(),
        vec![MOCK_WALLET_CLIENT_ID.to_string()],
        config_params,
        Some(WiaConfig {
            wia_trust_anchors: trust_anchors.clone(),
        }),
        sessions,
        MemoryNonceStore::new(),
    )
    .unwrap();

    (issuer, trust_anchors, wia_keypair)
}

pub fn setup_mock_authorizing_issuer<G>(
    issuer_identifier: IssuerIdentifier,
    attestation_count: NonZeroUsize,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
    flow: StaticAuthorizationCodeFlow,
) -> (MockAuthorizingIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let par_store = MemoryStore::new(PAR_TTL);
    let (issuer, trust_anchor, wia_keypair) = setup_mock_issuer(issuer_identifier, attestation_count, sessions);
    let authorizing_issuer = AuthorizingIssuer::new(Arc::new(issuer), par_store, flow);

    (authorizing_issuer, trust_anchor, wia_keypair)
}
