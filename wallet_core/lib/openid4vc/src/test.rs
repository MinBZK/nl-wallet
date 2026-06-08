use std::collections::HashMap;
use std::convert::Infallible;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;

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

use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::authorization_code_flow::WalletAuthorizationContext;
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
use crate::server_state::MemorySessionStore;
use crate::server_state::SessionStore;
use crate::store::MemoryStore;
use crate::token::AuthorizationCode;

pub const MOCK_ATTESTATION_TYPES: [&str; 2] = ["com.example.pid", "com.example.address"];
pub const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

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

/// Test-only implementation of [`AuthorizationCodeFlow`] for the auth-code path. `authorize`
/// captures the wallet's [`WalletAuthorizationContext`] into a single-slot cell and redirects to a
/// dummy upstream URL. Tests do not follow that redirect; they call
/// [`StaticAuthorizationCodeFlow::fake_complete_authorization`] directly to plant the corresponding
/// `AuthCodeIssued` session, standing in for what a real upstream callback would do.
pub struct StaticAuthorizationCodeFlow {
    documents: VecNonEmpty<IssuableDocument>,
    captured: Mutex<Option<WalletAuthorizationContext>>,
}

impl StaticAuthorizationCodeFlow {
    pub fn new(documents: VecNonEmpty<IssuableDocument>) -> Self {
        Self {
            documents,
            captured: Mutex::new(None),
        }
    }

    /// Test-only stand-in for the real upstream callback. Reads the context captured during
    /// `authorize` and hands it to [`AuthorizingIssuer::complete_authorization`], which generates the
    /// issuer-side authorization code and writes the `AuthCodeIssued` session. Returns the generated code
    /// so the test can drive `/token`.
    pub async fn fake_complete_authorization<K, L, S, N, PAS>(
        &self,
        authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, Self>,
    ) -> AuthorizationCode
    where
        S: SessionStore<IssuanceData>,
    {
        let context = self
            .captured
            .lock()
            .unwrap()
            .take()
            .expect("fake_complete_authorization called before /authorize was hit");

        let redirect_url = authorizing_issuer
            .complete_authorization(self.documents.clone(), context)
            .await
            .unwrap();

        // Extract the authorization code from the redirect URL.
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        params.get("code").unwrap().clone().into()
    }
}

impl AuthorizationCodeFlow for StaticAuthorizationCodeFlow {
    type Error = Infallible;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        *self.captured.lock().expect("mutex shouldn't be poisoned in test") = Some(context);

        // The wallet would be redirected to the upstream provider here. Tests drive the upstream
        // callback directly via `fake_complete_authorization`, so this redirect is never followed and
        // its target is never inspected.
        Ok(AuthorizeOutcome::RedirectTo(
            "https://upstream.example.com/oauth2/authorize"
                .parse()
                .expect("valid URL"),
        ))
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
    wallet_redirect_uris: VecNonEmpty<Url>,
) -> (MockAuthorizingIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let par_store = MemoryStore::new(PAR_TTL);
    let (issuer, trust_anchor, wia_keypair) = setup_mock_issuer(issuer_identifier, attestation_count, sessions);
    let authorizing_issuer = AuthorizingIssuer::new(Arc::new(issuer), par_store, flow, wallet_redirect_uris);

    (authorizing_issuer, trust_anchor, wia_keypair)
}
