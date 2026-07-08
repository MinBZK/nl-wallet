use std::collections::HashSet;
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
use derive_more::Constructor;
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

use crate::AuthorizationErrorCode;
use crate::ErrorWithCode;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::authorization_code_flow::WalletAuthorizationContext;
use crate::authorizing_issuer::AuthorizingIssuer;
use crate::credential_configurations::CredentialConfigurationParameters;
use crate::issuable_document::CredentialKind;
use crate::issuable_document::IssuableDocument;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::issuer_identifier::IssuerIdentifier;
use crate::mock::MOCK_WALLET_CLIENT_ID;
use crate::nonce::memory_store::MemoryNonceStore;
use crate::par::PAR_TTL;
use crate::server_state::MemorySessionStore;
use crate::store::MemoryStore;

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
    StaticAuthorizingFlow,
>;

fn mock_claims(required_attr: &str) -> Vec<ClaimMetadata> {
    MOCK_ATTRS
        .iter()
        .map(|(key, _)| ClaimMetadata {
            path: vec_nonempty![ClaimPath::SelectByKey(key.to_string())],
            display: vec![ClaimDisplayMetadata {
                locale: "en".to_string(),
                label: key.to_string(),
                description: None,
            }],
            sd: ClaimSelectiveDisclosureMetadata::Allowed,
            mandatory: *key == required_attr,
            svg_id: None,
        })
        .collect()
}

pub fn mock_type_metadata(vct: &str) -> TypeMetadata {
    TypeMetadata::try_new(UncheckedTypeMetadata {
        vct: vct.to_string(),
        claims: mock_claims(""),
        ..UncheckedTypeMetadata::empty_example()
    })
    .unwrap()
}

/// Like [`mock_type_metadata`], but with a JSON schema that marks `required_attr` as a required
/// property; the other [`MOCK_ATTRS`] stay optional. Issuing a document that lacks `required_attr`
/// then fails schema validation at credential issuance.
pub fn mock_type_metadata_with_required_attr(vct: &str, required_attr: &str) -> TypeMetadata {
    TypeMetadata::try_new(UncheckedTypeMetadata {
        vct: vct.to_string(),
        claims: mock_claims(required_attr),
        ..UncheckedTypeMetadata::empty_example()
    })
    .unwrap()
}

/// A single mock document for `attestation_type` carrying exactly the given attributes
/// (typically a subset of [`MOCK_ATTRS`]).
pub fn mock_issuable_document_with_attrs(attestation_type: &str, attrs: &[(&str, &str)]) -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(
        CredentialKind::new(Format::SdJwt, attestation_type.to_string()),
        IndexMap::from_iter(attrs.iter().map(|(key, val)| {
            (
                key.to_string(),
                Attribute::Single(AttributeValue::Text(val.to_string())),
            )
        }))
        .into(),
    )
    .unwrap()
}

pub fn mock_issuable_documents(document_count: NonZeroUsize) -> VecNonEmpty<IssuableDocument> {
    (0..document_count.get())
        .map(|i| mock_issuable_document_with_attrs(MOCK_ATTESTATION_TYPES[i], &MOCK_ATTRS))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

/// Test-only implementation of [`AuthorizationCodeFlow`] for the auth-code path. `authorize` either returns
/// [`AuthorizeOutcome::Authorized`] with the preconfigured documents, so the `openid4vc` layer immediately writes the
/// `AuthCodeIssued` session and redirects the user-agent straight back to the wallet with the issuer-generated code, or
/// it returns an error that converts to [`AuthorizationErrorCode`], which is also encoded in a redirect back to the
/// wallet.
#[derive(derive_more::Constructor)]
pub struct StaticAuthorizingFlow {
    authorize_result: Result<VecNonEmpty<IssuableDocument>, AuthorizationErrorCode>,
}

#[derive(Debug, thiserror::Error, Constructor)]
#[error("StaticAuthorizingFlowError")]
pub struct StaticAuthorizingFlowError(AuthorizationErrorCode);

impl ErrorWithCode for StaticAuthorizingFlowError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        let Self(inner) = self;

        inner.clone()
    }
}

impl AuthorizationCodeFlow for StaticAuthorizingFlow {
    type Error = StaticAuthorizingFlowError;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        let documents = self.authorize_result.clone().map_err(StaticAuthorizingFlowError)?;

        let outcome = AuthorizeOutcome::Authorized(documents.clone(), Box::new(context));

        Ok(outcome)
    }
}

/// Create a mock [`Issuer`] based on an [`IssuerIdentifier`] with a shared session store and some default credential
/// configurations.
pub fn setup_mock_issuer<G>(
    issuer_identifier: IssuerIdentifier,
    attestation_count: NonZeroUsize,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
) -> (MockIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let type_metadata = MOCK_ATTESTATION_TYPES[..attestation_count.get()]
        .iter()
        .map(|attestation_type| mock_type_metadata(attestation_type))
        .collect();

    setup_mock_issuer_from_sd_jwt_metadata(issuer_identifier, type_metadata, sessions)
}

/// Create a mock [`Issuer`] based on an [`IssuerIdentifier`] and a shared session store. Its credential configurations
/// are are all in SD-JWT format and based on a single SD-JWT VC Type Metadata document.
pub fn setup_mock_issuer_from_sd_jwt_metadata<G>(
    issuer_identifier: IssuerIdentifier,
    metadata: Vec<TypeMetadata>,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
) -> (MockIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let attestations = metadata
        .into_iter()
        .map(|metadata| {
            let (attestation_type, _, metadata_documents) = TypeMetadataDocuments::from_single_example(metadata);

            (Format::SdJwt, attestation_type, metadata_documents)
        })
        .collect();

    setup_mock_issuer_attestation_types_and_metadata(issuer_identifier, attestations, sessions)
}

/// Create a mock [`Issuer`] based on an [`IssuerIdentifier`] and a shared session store. Its credential configurations
/// are based on a list of format / attestation type combinations and the relevant SD-JWT VC Type Metadata documents.
pub fn setup_mock_issuer_attestation_types_and_metadata<G>(
    issuer_identifier: IssuerIdentifier,
    attestations: Vec<(Format, String, TypeMetadataDocuments)>,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
) -> (MockIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let metadata_keypair = ca.generate_wrpac_issuer_mock().unwrap();
    let issuance_keypair = generate_issuer_mock_with_registration(&ca, &IssuerRegistration::new_mock()).unwrap();
    let trust_anchors = TrustAnchors::from(&ca);
    let wia_keypair = ca.generate_wia_mock().unwrap();

    let config_params = attestations
        .into_iter()
        .map(|(format, attestation_type, metadata_documents)| {
            let config_id = format!("{attestation_type}_{format}");

            let mut status_list = MockStatusListService::new();
            let status_list_uri_path = config_id.replace(':', "-");
            status_list
                .expect_obtain_status_claims()
                .returning(move |_, _, copies| {
                    let uri = format!("https://tsl.example.com/{status_list_uri_path}")
                        .parse()
                        .unwrap();
                    Ok(generate_status_claims(&uri, copies))
                });
            status_list
                .expect_start_refresh_job()
                .return_once(|| tokio::task::spawn(async {}).abort_handle());

            let params = CredentialConfigurationParameters {
                credential_kind: CredentialKind::new(format, attestation_type),
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

            (config_id.into(), params)
        })
        .collect();

    let issuer = MockIssuer::try_new(
        issuer_identifier,
        metadata_keypair,
        NonZeroU8::new(4).unwrap(),
        HashSet::from([MOCK_WALLET_CLIENT_ID.to_string()]),
        config_params,
        trust_anchors.clone(),
        sessions,
        MemoryNonceStore::new(),
    )
    .unwrap();

    (issuer, trust_anchors, wia_keypair)
}

/// Create a mock [`AuthorizingIssuer`] based on an [`IssuerIdentifier`] and a shared session store. Its credential
/// configurations are are all in SD-JWT format and based on a single SD-JWT VC Type Metadata document.
pub fn setup_mock_authorizing_issuer_from_sd_jwt_metadata<G>(
    issuer_identifier: IssuerIdentifier,
    type_metadata: Vec<TypeMetadata>,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
    flow: StaticAuthorizingFlow,
    wallet_redirect_uris: VecNonEmpty<Url>,
) -> (MockAuthorizingIssuer<G>, TrustAnchors, KeyPair)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let par_store = MemoryStore::new(PAR_TTL);
    let (issuer, trust_anchor, wia_keypair) =
        setup_mock_issuer_from_sd_jwt_metadata(issuer_identifier, type_metadata, sessions);
    let authorizing_issuer = AuthorizingIssuer::new(Arc::new(issuer), par_store, flow, wallet_redirect_uris);

    (authorizing_issuer, trust_anchor, wia_keypair)
}
