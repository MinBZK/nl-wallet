use serde::Deserialize;
use serde::Serialize;
use utils::vec_at_least::VecNonEmpty;

use crate::token::CredentialPreview;

/// Credential Preview Response as per the OpenID4VCI profile specification.
///
/// Returned as a JSON-encoded body in an HTTP response to the credential preview endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPreviewResponse {
    /// One or more Credential Previews.
    pub credential_previews: VecNonEmpty<CredentialPreview>,
}
