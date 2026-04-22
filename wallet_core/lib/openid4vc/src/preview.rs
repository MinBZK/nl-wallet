use serde::Deserialize;
use serde::Serialize;
use utils::vec_at_least::VecNonEmpty;

use crate::token::CredentialPreview;

/// Credential Preview Request as per the OpenID4VCI profile specification.
///
/// Sent as a JSON-encoded body in an HTTP POST to the credential preview endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialPreviewRequest {
    /// REQUIRED when an Authorization Details of type `openid_credential` was returned from the Token Response.
    /// An array of `credential_identifier` strings that identify the Credential Datasets to preview.
    /// MUST NOT be used when `credential_configuration_ids` is present.
    CredentialIdentifiers {
        credential_identifiers: VecNonEmpty<String>,
    },

    /// REQUIRED if `credential_identifiers` was not returned from the Token Response.
    /// An array of strings, each uniquely identifying a key in `credential_configurations_supported`.
    /// MUST NOT be used when `credential_identifiers` is present.
    CredentialConfigurationIds {
        credential_configuration_ids: VecNonEmpty<String>,
    },
}

/// Credential Preview Response as per the OpenID4VCI profile specification.
///
/// Returned as a JSON-encoded body in an HTTP response to the credential preview endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPreviewResponse {
    /// One or more Credential Previews.
    pub credential_previews: VecNonEmpty<CredentialPreview>,
}
