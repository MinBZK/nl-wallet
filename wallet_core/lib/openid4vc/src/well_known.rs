use serde::de::DeserializeOwned;
use url::Url;

use crate::issuer_identifier::IssuerIdentifier;
use crate::oidc::HttpJsonClient;

pub enum WellKnownPath {
    CredentialIssuer,
    OauthAuthorizationServer,
    OpenidConfiguration,
}

impl WellKnownPath {
    fn as_str(&self) -> &'static str {
        match self {
            Self::CredentialIssuer => "openid-credential-issuer",
            Self::OauthAuthorizationServer => "oauth-authorization-server",
            Self::OpenidConfiguration => "openid-configuration",
        }
    }
}

pub trait WellKnownMetadata {
    fn issuer_identifier(&self) -> &IssuerIdentifier;
}

#[derive(Debug, thiserror::Error)]
pub enum WellKnownError {
    #[error("could not fetch or deserialize well-known metadata: {0}")]
    Http(#[from] reqwest::Error),

    #[error("issuer identifier in well-known metadata does not match, expected: {expected}, received: {received}")]
    IssuerIdentifierMismatch {
        expected: Box<IssuerIdentifier>,
        received: Box<IssuerIdentifier>,
    },
}

/// Constructs a well-known metadata URL by appending the well-known path to the issuer identifier.
fn well_known_url(issuer: &IssuerIdentifier, suffix: &str) -> Url {
    // TODO (PVW-5527): spec-compliant URL construction (inserting the well-known path between host and path
    // components as per the OpenID4VCI specification) is tracked in PVW-5527.
    issuer.as_base_url().join(&format!("/.well-known/{suffix}"))
}

pub async fn fetch_well_known_unvalidated<T>(
    client: &HttpJsonClient,
    issuer: &IssuerIdentifier,
    path: WellKnownPath,
) -> Result<T, WellKnownError>
where
    T: DeserializeOwned,
{
    let url = well_known_url(issuer, path.as_str());
    let metadata: T = client.get(url).await?;
    Ok(metadata)
}

pub async fn fetch_well_known<T>(
    client: &HttpJsonClient,
    issuer: &IssuerIdentifier,
    path: WellKnownPath,
) -> Result<T, WellKnownError>
where
    T: DeserializeOwned + WellKnownMetadata,
{
    let url = well_known_url(issuer, path.as_str());
    let metadata: T = client.get(url).await?;
    if metadata.issuer_identifier() != issuer {
        return Err(WellKnownError::IssuerIdentifierMismatch {
            expected: Box::new(issuer.clone()),
            received: Box::new(metadata.issuer_identifier().clone()),
        });
    }
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issuer(s: &str) -> IssuerIdentifier {
        s.parse().unwrap()
    }

    #[test]
    fn test_well_known_url_no_path() {
        let url = well_known_url(
            &issuer("https://example.com/"),
            WellKnownPath::CredentialIssuer.as_str(),
        );
        assert_eq!(url.as_str(), "https://example.com/.well-known/openid-credential-issuer");
    }

    #[test]
    fn test_well_known_url_with_path() {
        // Note: spec-compliant behavior would insert the well-known path before the tenant segment
        // (PVW-5527). This test documents the current (non-compliant) behavior.
        let url = well_known_url(
            &issuer("https://example.com/tenant"),
            WellKnownPath::CredentialIssuer.as_str(),
        );
        assert_eq!(
            url.as_str(),
            "https://example.com/tenant/.well-known/openid-credential-issuer"
        );
    }

    #[test]
    fn test_well_known_url_oauth_suffix() {
        let url = well_known_url(
            &issuer("https://example.com/"),
            WellKnownPath::OauthAuthorizationServer.as_str(),
        );
        assert_eq!(
            url.as_str(),
            "https://example.com/.well-known/oauth-authorization-server"
        );
    }
}
