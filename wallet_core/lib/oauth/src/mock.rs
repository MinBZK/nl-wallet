use indexmap::IndexSet;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

impl AuthorizationServerMetadata {
    /// Construct a new `AuthorizationServerMetadata` based on the OP's URL and some standardized or reasonable
    /// defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();
        let auth_url = issuer_url.join("/authorize");
        let token_url = issuer_url.join("/issuance/token");
        let challenge_url = issuer_url.join("/issuance/client_auth_challenge");
        let jwks_url = issuer_url.join("/jwks.json");
        let par_url = issuer_url.join("/par");

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
            pushed_authorization_request_endpoint: Some(par_url),
            challenge_endpoint: Some(challenge_url),

            ..AuthorizationServerMetadata::new(issuer_identifier, token_url)
        }
    }
}
