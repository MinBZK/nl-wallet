use std::collections::HashMap;
use std::iter;

use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use dcql::disclosure::ExtendingVctRetriever;

use crate::issuance_session::CredentialWithMetadata;
use crate::issuance_session::HttpVcMessageClient;
use crate::issuance_session::IssuanceSession;
use crate::issuance_session::IssuanceSessionError;
use crate::issuance_session::NormalizedCredentialPreview;
use crate::issuer_identifier::IssuerIdentifier;
use crate::issuer_metadata::BatchCredentialIssuance;
use crate::issuer_metadata::IssuerMetadata;
use crate::issuer_metadata::NonZeroOrOneU64;
use crate::oidc::Config;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;

// Re-exported for convenience
pub use wscd::mock_remote::MOCK_WALLET_CLIENT_ID;

// We can't use `mockall::automock!` on the `IssuerClient` trait directly since `automock` doesn't accept
// traits using generic methods, and "impl trait" arguments, so we use `mockall::mock!` to make an indirection.

mockall::mock! {
    #[derive(Debug)]
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
        _: IssuerIdentifier,
        _: TokenRequest,
        _: &[TrustAnchor<'_>],
    ) -> Result<Self, IssuanceSessionError>
    where
        Self: Sized,
    {
        Self::start()
    }

    async fn accept_issuance<W>(
        &self,
        _: &[TrustAnchor<'_>],
        _: &W,
        _: bool,
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

pub struct ExtendingVctRetrieverStub;
impl ExtendingVctRetriever for ExtendingVctRetrieverStub {
    fn retrieve(&self, _vct_value: &str) -> impl Iterator<Item = &str> {
        iter::empty()
    }
}

impl Config {
    /// Construct a new `Config` based on the OP's URL and some standardized or reasonable defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();
        let auth_url = issuer_url.join("/authorize");
        let token_url = issuer_url.join("/issuance/token");
        let jwks_url = issuer_url.join("/jwks.json");

        Self {
            userinfo_endpoint: Some(issuer_url.join("/userinfo")),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),

            ..Config::new(issuer_identifier, auth_url, token_url, jwks_url)
        }
    }
}

impl IssuerMetadata {
    pub fn new_mock(issuer_identifier: IssuerIdentifier, attestation_type: &str) -> IssuerMetadata {
        let credential_endpoint = issuer_identifier.join_issuer_url("/issuance/credential");
        let batch_credential_endpoint = issuer_identifier.join_issuer_url("/issuance/batch_credential");
        let credential_preview_endpoint = issuer_identifier
            .join_issuer_url("/issuance/credential_preview");

        IssuerMetadata {
            credential_issuer: issuer_identifier,
            authorization_servers: None,
            credential_endpoint,
            batch_credential_endpoint: Some(batch_credential_endpoint),
            nonce_endpoint: None,
            deferred_credential_endpoint: None,
            notification_endpoint: None,
            credential_request_encryption: None,
            credential_response_encryption: None,
            batch_credential_issuance: Some(BatchCredentialIssuance {
                batch_size: NonZeroOrOneU64::try_new(10.try_into().unwrap()).unwrap(),
            }),
            display: None,
            credential_configurations_supported: HashMap::new(),
            credential_preview_endpoint: Some(credential_preview_endpoint),
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
