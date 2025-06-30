use std::collections::HashMap;

use chrono::Utc;
use indexmap::IndexMap;
use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::factory::KeyFactory;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredential;
use jwt::wte::WteClaims;
use mdoc::holder::Mdoc;
use mdoc::test::TestDocument;
use mdoc::verifier::ItemsRequests;
use mdoc::IssuerSigned;
use mdoc::ItemsRequest;

use crate::issuance_session::CredentialWithMetadata;
use crate::issuance_session::HttpVcMessageClient;
use crate::issuance_session::IssuanceSession;
use crate::issuance_session::IssuanceSessionError;
use crate::issuance_session::NormalizedCredentialPreview;
use crate::metadata::CredentialResponseEncryption;
use crate::metadata::IssuerData;
use crate::metadata::IssuerMetadata;
use crate::oidc::Config;
use crate::openid4vp::FormatAlg;
use crate::openid4vp::VpFormat;
use crate::presentation_exchange::Constraints;
use crate::presentation_exchange::Field;
use crate::presentation_exchange::InputDescriptor;
use crate::presentation_exchange::LimitDisclosure;
use crate::presentation_exchange::PdConversionError;
use crate::presentation_exchange::PresentationDefinition;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;

// Re-exported for convenience
pub use poa::factory::mock::MOCK_WALLET_CLIENT_ID;

// We can't use `mockall::automock!` on the `IssuerClient` trait directly since `automock` doesn't accept
// traits using generic methods, and "impl trait" arguments, so we use `mockall::mock!` to make an indirection.

mockall::mock! {
    pub IssuanceSession {
        pub fn start() -> Result<Self, IssuanceSessionError>
        where
            Self: Sized;

        pub fn accept(
            &self,
        ) -> Result<Vec<CredentialWithMetadata>, IssuanceSessionError>;

        pub fn reject(self) -> Result<(), IssuanceSessionError>;

        pub fn normalized_credential_previews(&self) -> &[NormalizedCredentialPreview];

        pub fn issuer(&self) -> &IssuerRegistration;
    }
}

impl IssuanceSession for MockIssuanceSession {
    async fn start_issuance(
        _: HttpVcMessageClient,
        _: BaseUrl,
        _: TokenRequest,
        _: &[TrustAnchor<'_>],
    ) -> Result<Self, IssuanceSessionError>
    where
        Self: Sized,
    {
        Self::start()
    }

    async fn accept_issuance<K, KF>(
        &self,
        _: &[TrustAnchor<'_>],
        _: &KF,
        _: Option<JwtCredential<WteClaims>>,
    ) -> Result<Vec<CredentialWithMetadata>, IssuanceSessionError> {
        self.accept()
    }

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError> {
        self.reject()
    }

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview] {
        self.normalized_credential_previews()
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        self.issuer()
    }
}

impl Config {
    /// Construct a new `Config` based on the OP's URL and some standardized or reasonable defaults.
    pub fn new_mock(issuer: &BaseUrl) -> Self {
        Self {
            userinfo_endpoint: Some(issuer.join("/userinfo")),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),

            ..Config::new(
                issuer.clone(),
                issuer.join("/authorize"),
                issuer.join("/token"),
                issuer.join("/jwks.json"),
            )
        }
    }
}

impl IssuerMetadata {
    pub fn new_mock(url: &BaseUrl) -> IssuerMetadata {
        IssuerMetadata {
            issuer_config: IssuerData {
                credential_issuer: url.clone(),
                authorization_servers: None,
                credential_endpoint: url.join_base_url("/credential"),
                batch_credential_endpoint: Some(url.join_base_url("/batch_credential")),
                deferred_credential_endpoint: None,
                notification_endpoint: None,
                credential_response_encryption: CredentialResponseEncryption {
                    alg_values_supported: vec![],
                    enc_values_supported: vec![],
                    encryption_required: false,
                },
                credential_identifiers_supported: None,
                display: None,
                credential_configurations_supported: HashMap::new(),
            },
            protected_metadata: None,
        }
    }
}

impl TokenRequest {
    pub fn new_mock() -> TokenRequest {
        TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: "123".to_string().into(),
            },
            code_verifier: None,
            client_id: None,
            redirect_uri: None,
        }
    }
}

pub async fn test_document_to_issuer_signed<KF>(doc: TestDocument, ca: &Ca, key_factory: &KF) -> (IssuerSigned, KF::Key)
where
    KF: KeyFactory,
{
    let key = key_factory.generate_new().await.unwrap();

    let issuer_signed = doc.issuer_signed(ca, &key, Utc::now()).await;
    (issuer_signed, key)
}

pub async fn test_document_to_mdoc<KF>(doc: TestDocument, ca: &Ca, key_factory: &KF) -> (Mdoc, KF::Key)
where
    KF: KeyFactory,
{
    let key = key_factory.generate_new().await.unwrap();

    let mdoc = doc.sign(ca, &key).await;
    (mdoc, key)
}

impl From<&ItemsRequests> for PresentationDefinition {
    fn from(items_requests: &ItemsRequests) -> Self {
        PresentationDefinition {
            id: random_string(16),
            input_descriptors: items_requests
                .0
                .iter()
                .map(|items_request| InputDescriptor {
                    id: items_request.doc_type.clone(),
                    format: VpFormat::MsoMdoc {
                        alg: IndexSet::from([FormatAlg::ES256]),
                    },
                    constraints: Constraints {
                        limit_disclosure: LimitDisclosure::Required,
                        fields: items_request
                            .name_spaces
                            .iter()
                            .flat_map(|(namespace, attrs)| {
                                attrs.iter().map(|(attr, intent_to_retain)| Field {
                                    path: vec![format!("$['{}']['{}']", namespace.as_str(), attr.as_str())],
                                    intent_to_retain: *intent_to_retain,
                                })
                            })
                            .collect(),
                    },
                })
                .collect(),
        }
    }
}

impl TryFrom<&PresentationDefinition> for ItemsRequests {
    type Error = PdConversionError;

    fn try_from(pd: &PresentationDefinition) -> Result<Self, Self::Error> {
        let items_requests = pd
            .input_descriptors
            .iter()
            .map(|input_descriptor| {
                let VpFormat::MsoMdoc { alg } = &input_descriptor.format;
                if !alg.contains(&FormatAlg::ES256) {
                    return Err(PdConversionError::UnsupportedAlgs);
                }

                let mut name_spaces: IndexMap<String, IndexMap<String, bool>> = IndexMap::new();
                for field in &input_descriptor.constraints.fields {
                    let (namespace, attr) = field.parse_paths()?;
                    name_spaces
                        .entry(namespace)
                        .or_default()
                        .insert(attr, field.intent_to_retain);
                }

                Ok(ItemsRequest {
                    doc_type: input_descriptor.id.clone(),
                    request_info: None,
                    name_spaces,
                })
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        Ok(items_requests.into())
    }
}
