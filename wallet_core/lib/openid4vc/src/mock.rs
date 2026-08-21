use std::iter;

use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use dcql::disclosure::ExtendingVctRetriever;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::token::AuthorizationCode;
pub use wscd::mock_remote::MOCK_WALLET_CLIENT_ID;

use crate::metadata::issuer_metadata::AtLeastTwoU64;
// Re-exported for convenience
use crate::metadata::issuer_metadata::BatchCredentialIssuance;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::ProofType;
use crate::token::VciTokenRequest;

pub struct ExtendingVctRetrieverStub;
impl ExtendingVctRetriever for ExtendingVctRetrieverStub {
    fn retrieve(&self, _vct_value: &str) -> impl Iterator<Item = &str> {
        iter::empty()
    }
}

impl IssuerMetadata {
    pub fn new_mock(
        issuer_identifier: IssuerIdentifier,
        credential_configs: Vec<(CredentialConfigurationId, CredentialKind)>,
    ) -> IssuerMetadata {
        let issuer_url = issuer_identifier.as_issuer_url();
        let credential_endpoint = issuer_url.join_issuer_url("/issuance/credential");
        let batch_credential_endpoint = issuer_url.join_issuer_url("/issuance/batch_credential");
        let nonce_endpoint = issuer_url.join_issuer_url("/issuance/nonce");
        let credential_preview_endpoint = issuer_url.join_issuer_url("/issuance/credential_preview");

        let credential_configurations_supported = credential_configs
            .into_iter()
            .map(|(config_id, credential_kind)| {
                let scope = format!("{config_id}_scope").parse().unwrap();
                let type_metadata_uri = issuer_url
                    .join_issuer_url("/issuance/type_metadata")
                    .join_config_id(config_id.as_ref());

                let config = match credential_kind.format {
                    Format::MsoMdoc => CredentialConfiguration::new_mdoc_ecdsa_p256_sha256(
                        credential_kind.attestation_type,
                        scope,
                        vec![ProofType::Jwt],
                        vec![],
                        vec![],
                        type_metadata_uri,
                    ),
                    Format::SdJwt => CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                        credential_kind.attestation_type,
                        scope,
                        vec![ProofType::Jwt],
                        vec![],
                        vec![],
                        type_metadata_uri,
                    ),
                };

                (config_id, config)
            })
            .collect();

        IssuerMetadata {
            credential_issuer: issuer_identifier,
            authorization_servers: None,
            endpoints: IssuerEndpoints {
                credential_endpoint,
                batch_credential_endpoint: Some(batch_credential_endpoint),
                nonce_endpoint: Some(nonce_endpoint),
                deferred_credential_endpoint: None,
                notification_endpoint: None,
                credential_preview_endpoint: Some(credential_preview_endpoint),
            },
            credential_request_encryption: None,
            credential_response_encryption: None,
            batch_credential_issuance: Some(BatchCredentialIssuance {
                batch_size: AtLeastTwoU64::try_new(10.try_into().unwrap()).unwrap(),
            }),
            display: None,
            credential_configurations_supported,
        }
    }
}

impl VciTokenRequest {
    pub fn new_mock() -> Self {
        Self::new_mock_with_pre_authorized_code("123".to_string().into())
    }

    pub fn new_mock_with_pre_authorized_code(pre_authorized_code: AuthorizationCode) -> Self {
        VciTokenRequest::new_pre_authorized(pre_authorized_code)
    }
}
