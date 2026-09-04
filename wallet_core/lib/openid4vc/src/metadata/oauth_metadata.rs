use oauth::issuer_identifier::IssuerIdentifier;
use oauth::metadata::oauth_metadata::AuthorizationServerMetadata;
use oauth::metadata::oauth_metadata::ClientAttestationMetadataExtension;
use oauth::metadata::oauth_metadata::ParMetadataExtension;
use oauth::metadata::well_known::WellKnownMetadata;
use serde::Deserialize;
use serde::Serialize;

/// OAuth 2.0 Authorization Server Metadata as served and consumed for issuance by this Issuer, published at
/// `.well-known/oauth-authorization-server`. Extends RFC 8414 [`AuthorizationServerMetadata`] with the Pushed
/// Authorization Requests fields defined by RFC 9126 and the fields used for Attestation-Based Client
/// Authentication.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IssuerAuthorizationServerMetadata {
    #[serde(flatten)]
    pub oauth_metadata: AuthorizationServerMetadata,
    #[serde(flatten)]
    pub par_metadata_extension: ParMetadataExtension,
    #[serde(flatten)]
    pub client_attestation_metadata_extension: ClientAttestationMetadataExtension,
}

impl WellKnownMetadata for IssuerAuthorizationServerMetadata {
    const PATH: &'static str = AuthorizationServerMetadata::PATH;

    fn issuer_identifier(&self) -> &IssuerIdentifier {
        self.oauth_metadata.issuer_identifier()
    }
}

#[cfg(any(test, feature = "mock"))]
impl IssuerAuthorizationServerMetadata {
    /// Construct a new `IssuerAuthorizationServerMetadata` based on the Issuer's URL and some standardized or
    /// reasonable defaults.
    pub fn new_mock(issuer_identifier: IssuerIdentifier) -> Self {
        let issuer_url = issuer_identifier.as_base_url();
        let par_url = issuer_url.join("/par");
        let challenge_url = issuer_url.join("/issuance/client_auth_challenge");

        Self {
            oauth_metadata: AuthorizationServerMetadata::new_mock(issuer_identifier),
            par_metadata_extension: ParMetadataExtension {
                pushed_authorization_request_endpoint: Some(par_url),
                require_pushed_authorization_requests: false,
            },
            client_attestation_metadata_extension: ClientAttestationMetadataExtension {
                challenge_endpoint: Some(challenge_url),
                ..ClientAttestationMetadataExtension::default()
            },
        }
    }
}
