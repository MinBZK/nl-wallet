use std::collections::HashMap;

use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;

use crypto::factory::KeyFactory;
use crypto::keys::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredential;
use jwt::wte::WteClaims;
use poa::factory::PoaFactory;

use crate::issuance_session::CredentialPreviewsNormalizedMetadata;
use crate::issuance_session::HttpVcMessageClient;
use crate::issuance_session::IssuanceSession;
use crate::issuance_session::IssuanceSessionError;
use crate::issuance_session::IssuedCredentialCopies;
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
        pub fn start() -> Result<(Self, CredentialPreviewsNormalizedMetadata), IssuanceSessionError>
        where
            Self: Sized;

        pub fn accept(
            &self,
        ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>;

        pub fn reject(self) -> Result<(), IssuanceSessionError>;
    }
}

impl IssuanceSession for MockIssuanceSession {
    async fn start_issuance(
        _: HttpVcMessageClient,
        _: BaseUrl,
        _: TokenRequest,
        _: &[TrustAnchor<'_>],
    ) -> Result<(Self, CredentialPreviewsNormalizedMetadata), IssuanceSessionError>
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
        _: BaseUrl,
    ) -> Result<Vec<IssuedCredentialCopies>, IssuanceSessionError>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
        KF: PoaFactory<Key = K>,
    {
        self.accept()
    }

    async fn reject_issuance(self) -> Result<(), IssuanceSessionError> {
        self.reject()
    }
}

impl Config {
    /// Construct a new `Config` based on the OP's URL and some standardized or reasonable defaults.
    pub fn new_mock(issuer: &BaseUrl) -> Self {
        Self {
            issuer: issuer.clone(),
            authorization_endpoint: issuer.join("/authorize"),
            token_endpoint: issuer.join("/token"),
            userinfo_endpoint: Some(issuer.join("/userinfo")),
            jwks_uri: issuer.join("/jwks.json"),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),
            response_modes_supported: None,
            grant_types_supported: None,
            acr_values_supported: None,
            subject_types_supported: IndexSet::new(),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),
            id_token_encryption_alg_values_supported: None,
            id_token_encryption_enc_values_supported: None,
            userinfo_signing_alg_values_supported: None,
            userinfo_encryption_alg_values_supported: None,
            userinfo_encryption_enc_values_supported: None,
            request_object_signing_alg_values_supported: None,
            request_object_encryption_alg_values_supported: None,
            request_object_encryption_enc_values_supported: None,
            token_endpoint_auth_methods_supported: None,
            token_endpoint_auth_signing_alg_values_supported: None,
            display_values_supported: None,
            claim_types_supported: None,
            claims_supported: None,
            service_documentation: None,
            claims_locales_supported: None,
            ui_locales_supported: None,
            claims_parameter_supported: false,
            request_parameter_supported: false,
            request_uri_parameter_supported: false,
            require_request_uri_registration: false,
            op_policy_uri: None,
            op_tos_uri: None,
            code_challenge_methods_supported: None,
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
