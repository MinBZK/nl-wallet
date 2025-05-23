use std::collections::HashMap;

use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredential;
use jwt::wte::WteClaims;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

use crate::issuance_session::HttpVcMessageClient;
use crate::issuance_session::IssuanceSession;
use crate::issuance_session::IssuanceSessionError;
use crate::issuance_session::IssuedCredentialCopies;
use crate::issuance_session::NormalizedCredentialPreview;
use crate::metadata::CredentialResponseEncryption;
use crate::metadata::IssuerData;
use crate::metadata::IssuerMetadata;
use crate::oidc::Config;
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
        ) -> Result<Vec<(IssuedCredentialCopies, VerifiedTypeMetadataDocuments)>, IssuanceSessionError>;

        pub fn reject(self) -> Result<(), IssuanceSessionError>;

        pub fn credential_preview_data(&self) -> &[NormalizedCredentialPreview];

        pub fn issuer(&self) -> Result<IssuerRegistration, IssuanceSessionError>;
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
    ) -> Result<Vec<(IssuedCredentialCopies, VerifiedTypeMetadataDocuments)>, IssuanceSessionError> {
        self.accept()
    }

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError> {
        self.reject()
    }

    fn credential_preview_data(&self) -> &[NormalizedCredentialPreview] {
        self.credential_preview_data()
    }

    fn issuer_registration(&self) -> Result<IssuerRegistration, IssuanceSessionError> {
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
