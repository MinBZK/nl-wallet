use std::collections::HashMap;
use std::iter;

use dcql::disclosure::ExtendingVctRetriever;
use indexmap::IndexSet;
pub use wscd::mock_remote::MOCK_WALLET_CLIENT_ID;

use crate::issuer_identifier::IssuerIdentifier;
// Re-exported for convenience
use crate::metadata::issuer_metadata::BatchCredentialIssuance;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::NonZeroOrOneU64;
use crate::metadata::issuer_metadata::ProofType;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;

pub struct ExtendingVctRetrieverStub;
impl ExtendingVctRetriever for ExtendingVctRetrieverStub {
    fn retrieve(&self, _vct_value: &str) -> impl Iterator<Item = &str> {
        iter::empty()
    }
}

impl AuthorizationServerMetadata {
    /// Construct a new `Config` based on the OP's URL and some standardized or reasonable defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();
        let auth_url = issuer_url.join("/authorize");
        let token_url = issuer_url.join("/issuance/token");
        let jwks_url = issuer_url.join("/jwks.json");

        Self {
            authorization_endpoint: Some(auth_url),
            jwks_uri: Some(jwks_url),
            userinfo_endpoint: Some(issuer_url.join("/userinfo")),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),

            ..AuthorizationServerMetadata::new(issuer_identifier, token_url)
        }
    }
}

impl IssuerMetadata {
    pub fn new_mock(issuer_identifier: IssuerIdentifier, attestation_type: &str) -> IssuerMetadata {
        let credential_endpoint = issuer_identifier.join_issuer_url("/issuance/credential");
        let batch_credential_endpoint = issuer_identifier.join_issuer_url("/issuance/batch_credential");
        let nonce_endpoint = issuer_identifier.join_issuer_url("/issuance/nonce");
        let credential_preview_endpoint = issuer_identifier.join_issuer_url("/issuance/credential_preview");

        IssuerMetadata {
            credential_issuer: issuer_identifier,
            authorization_servers: None,
            credential_endpoint,
            batch_credential_endpoint: Some(batch_credential_endpoint),
            nonce_endpoint: Some(nonce_endpoint),
            deferred_credential_endpoint: None,
            notification_endpoint: None,
            credential_request_encryption: None,
            credential_response_encryption: None,
            batch_credential_issuance: Some(BatchCredentialIssuance {
                batch_size: NonZeroOrOneU64::try_new(10.try_into().unwrap()).unwrap(),
            }),
            display: None,
            credential_configurations_supported: HashMap::from_iter(vec![(
                attestation_type.to_string(),
                CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                    attestation_type.to_string(),
                    vec![ProofType::Jwt],
                    vec![],
                    vec![],
                ),
            )]),
            credential_preview_endpoint: Some(credential_preview_endpoint),
        }
    }
}

impl TokenRequest {
    pub fn new_mock() -> TokenRequest {
        TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: "123".to_string().into(),
            },
            code_verifier: None,
            client_id: None,
            redirect_uri: None,
        }
    }
}
