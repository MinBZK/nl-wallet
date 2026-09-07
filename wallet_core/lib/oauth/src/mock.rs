use indexmap::IndexSet;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::metadata::oauth_metadata::OidcProviderMetadata;
use crate::metadata::oauth_metadata::OpenIdMetadataExtension;

impl AuthorizationServerMetadata {
    /// Construct a new `AuthorizationServerMetadata` based on the OP's URL and some standardized or reasonable
    /// defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();
        let auth_url = issuer_url.join("/authorize");
        let token_url = issuer_url.join("/issuance/token");
        let jwks_url = issuer_url.join("/jwks.json");

        Self {
            authorization_endpoint: Some(auth_url),
            jwks_uri: Some(jwks_url),
            registration_endpoint: None,
            scopes_supported: Some(IndexSet::from_iter(["openid".to_string()])),
            response_types_supported: IndexSet::from_iter(
                ["code", "code id_token", "id_token", "id_token token"].map(str::to_string),
            ),

            ..AuthorizationServerMetadata::new(issuer_identifier, token_url)
        }
    }
}

impl OpenIdMetadataExtension {
    /// Construct a new `OpenIdMetadataExtension` based on the OP's URL and some standardized or reasonable
    /// defaults.
    pub fn new_mock(issuer_identifier: &IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();

        Self {
            userinfo_endpoint: Some(issuer_url.join("/userinfo")),
            id_token_signing_alg_values_supported: IndexSet::from_iter(["RS256".to_string()]),
            ..OpenIdMetadataExtension::default()
        }
    }
}

impl OidcProviderMetadata {
    /// Construct a new `OidcProviderMetadata` based on the OP's URL and some standardized or reasonable defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let oidc_metadata_extension = OpenIdMetadataExtension::new_mock(&issuer_identifier);

        Self::new(
            AuthorizationServerMetadata::new_mock(issuer_identifier),
            oidc_metadata_extension,
        )
    }
}
