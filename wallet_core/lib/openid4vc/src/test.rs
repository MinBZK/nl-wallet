use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::issuable_document::IssuableDocument;
use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
use attestation_types::claim_path::ClaimPath;
use attestation_types::qualification::AttestationQualification;
use chrono::DateTime;
use chrono::Days;
use chrono::Utc;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use indexmap::IndexMap;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use rustls_pki_types::TrustAnchor;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::ClaimMetadata;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use token_status_list::status_list_service::mock::MockStatusListServices;
use token_status_list::status_list_service::mock::generate_status_claims;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::Format;
use crate::issuer::AttestationTypeConfig;
use crate::issuer::AttributeService;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::issuer::WuaConfig;
use crate::issuer_identifier::IssuerIdentifier;
use crate::mock::MOCK_WALLET_CLIENT_ID;
use crate::nonce::memory_store::MemoryNonceStore;
use crate::server_state::MemorySessionStore;
use crate::token::TokenRequest;

pub const MOCK_ATTESTATION_TYPES: [&str; 2] = ["com.example.pid", "com.example.address"];
pub const MOCK_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

pub type MockIssuer<G = TimeGenerator> =
    Issuer<SigningKey, MockAttrService, MemorySessionStore<IssuanceData, G>, MemoryNonceStore, MockStatusListServices>;

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

pub struct MockAttrService {
    pub documents: VecNonEmpty<IssuableDocument>,
}

impl AttributeService for MockAttrService {
    type Error = std::convert::Infallible;

    async fn attributes(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        Ok(self.documents.clone())
    }
}

pub fn setup_mock_issuer<G>(
    issuer_identifier: IssuerIdentifier,
    attr_service: MockAttrService,
    attestation_count: NonZeroUsize,
    sessions: Arc<MemorySessionStore<IssuanceData, G>>,
    upstream_oauth_identifier: Option<IssuerIdentifier>,
) -> (MockIssuer<G>, TrustAnchor<'static>, SigningKey)
where
    G: Generator<DateTime<Utc>> + Send + Sync + 'static,
{
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_keypair = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
    let trust_anchor = ca.to_trust_anchor().to_owned();
    let wua_issuer_privkey = SigningKey::random(&mut OsRng);

    let attestation_config = MOCK_ATTESTATION_TYPES[..attestation_count.get()]
        .iter()
        .map(|attestation_type| {
            let (_, _, metadata_documents) =
                TypeMetadataDocuments::from_single_example(mock_type_metadata(attestation_type));

            (
                attestation_type.to_string(),
                AttestationTypeConfig::try_new(
                    attestation_type,
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

    let mut status_list_service = MockStatusListServices::default();
    status_list_service
        .expect_obtain_status_claims()
        .returning(|attestation_type, _, _, copies| {
            let uri = format!("https://tsl.example.com/{}", attestation_type.replace(':', "-"))
                .parse()
                .unwrap();
            Ok(generate_status_claims(&uri, copies))
        });

    let issuer = MockIssuer::new(
        issuer_identifier,
        vec![MOCK_WALLET_CLIENT_ID.to_string()],
        attestation_config,
        Some(WuaConfig {
            wua_issuer_pubkey: wua_issuer_privkey.verifying_key().into(),
        }),
        upstream_oauth_identifier,
        attr_service,
        sessions,
        MemoryNonceStore::new(),
        Arc::new(status_list_service),
    );

    (issuer, trust_anchor, wua_issuer_privkey)
}
