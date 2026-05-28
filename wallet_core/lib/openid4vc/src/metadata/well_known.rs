use http_utils::reqwest::HttpJsonClient;
use serde::de::DeserializeOwned;
use url::Url;

use crate::issuer_identifier::IssuerIdentifier;

#[derive(Debug, Clone, Copy)]
pub enum WellKnownPath {
    CredentialIssuer,
    OauthAuthorizationServer,
    OpenidConfiguration,
}

impl WellKnownPath {
    fn as_str(self) -> &'static str {
        match self {
            Self::CredentialIssuer => "openid-credential-issuer",
            Self::OauthAuthorizationServer => "oauth-authorization-server",
            Self::OpenidConfiguration => "openid-configuration",
        }
    }

    fn url(self, issuer: &IssuerIdentifier) -> Url {
        let url = issuer.as_base_url();
        match self {
            Self::CredentialIssuer | Self::OauthAuthorizationServer => {
                let url = url.as_ref();
                let path = url.path();
                let path = path.strip_suffix('/').unwrap_or(path);
                url.join(&format!("/.well-known/{}{path}", self.as_str()))
                    .expect("both paths are already safe url encoded")
            }
            Self::OpenidConfiguration => {
                // OpenID Connect Discovery 1.0 and OAuth Authorization Server Metadata [RFC 8414]
                // conflict on this identifier: OIDC Discovery appends after the path, RFC 8414
                // inserts host-level. We follow OIDC Discovery because our only consumer of
                // `openid-configuration` is our OIDC provider (nl-rdo-max).
                issuer.as_base_url().join(&format!("/.well-known/{}", self.as_str()))
            }
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

pub async fn fetch_well_known<T>(
    client: &HttpJsonClient,
    issuer: &IssuerIdentifier,
    path: WellKnownPath,
) -> Result<T, WellKnownError>
where
    T: DeserializeOwned + WellKnownMetadata,
{
    let url = path.url(issuer);
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
    use rstest::rstest;

    use super::*;

    fn issuer(s: &str) -> IssuerIdentifier {
        s.parse().unwrap()
    }

    #[rstest]
    #[case(WellKnownPath::CredentialIssuer, "openid-credential-issuer")]
    #[case(WellKnownPath::OauthAuthorizationServer, "oauth-authorization-server")]
    #[case(WellKnownPath::OpenidConfiguration, "openid-configuration")]
    fn test_well_known_url_no_path(#[case] path: WellKnownPath, #[case] suffix: &str) {
        let issuer = issuer("https://example.com/");
        let url = path.url(&issuer);
        assert_eq!(url.as_str(), format!("https://example.com/.well-known/{suffix}"));
    }

    #[test]
    fn test_well_known_openid4ci_url_with_path() {
        let issuer = issuer("https://example.com/tenant");
        let url = WellKnownPath::CredentialIssuer.url(&issuer);
        assert_eq!(
            url.as_str(),
            "https://example.com/.well-known/openid-credential-issuer/tenant"
        );
    }

    #[test]
    fn test_well_known_openid4ci_url_with_path_and_trailing_slash() {
        let issuer = issuer("https://example.com/tenant/");
        let url = WellKnownPath::CredentialIssuer.url(&issuer);
        assert_eq!(
            url.as_str(),
            "https://example.com/.well-known/openid-credential-issuer/tenant"
        );
    }

    #[test]
    fn test_well_known_oauth_url_with_path() {
        let issuer = issuer("https://example.com/tenant");
        let url = WellKnownPath::OauthAuthorizationServer.url(&issuer);
        assert_eq!(
            url.as_str(),
            "https://example.com/.well-known/oauth-authorization-server/tenant"
        );
    }

    #[test]
    fn test_well_known_oidc_url_with_path() {
        let issuer = issuer("https://example.com/tenant");
        let url = WellKnownPath::OpenidConfiguration.url(&issuer);
        assert_eq!(
            url.as_str(),
            "https://example.com/tenant/.well-known/openid-configuration"
        );
    }
}
