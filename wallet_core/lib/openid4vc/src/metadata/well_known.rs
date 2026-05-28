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
        let url = issuer.as_base_url().as_ref();
        let path = url.path();
        let path = path.strip_suffix('/').unwrap_or(path);
        url.join(&format!("/.well-known/{}{path}", self.as_str()))
            .expect("both paths are already safe url encoded")
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
    fn test_well_known_url_no_path(#[case] path: WellKnownPath, #[case] identifier: &str) {
        let issuer = issuer("https://example.com/");
        let url = path.url(&issuer);
        assert_eq!(url.as_str(), format!("https://example.com/.well-known/{identifier}"));
    }

    #[rstest]
    #[case(WellKnownPath::CredentialIssuer, "openid-credential-issuer")]
    #[case(WellKnownPath::OauthAuthorizationServer, "oauth-authorization-server")]
    #[case(WellKnownPath::OpenidConfiguration, "openid-configuration")]
    fn test_well_known_url_with_path(#[case] path: WellKnownPath, #[case] identifier: &str) {
        let issuer = issuer("https://example.com/tenant");
        let url = path.url(&issuer);
        assert_eq!(
            url.as_str(),
            format!("https://example.com/.well-known/{identifier}/tenant")
        );
    }

    #[rstest]
    #[case(WellKnownPath::CredentialIssuer, "openid-credential-issuer")]
    #[case(WellKnownPath::OauthAuthorizationServer, "oauth-authorization-server")]
    #[case(WellKnownPath::OpenidConfiguration, "openid-configuration")]
    fn test_well_known_url_with_path_and_trailing_slash(#[case] path: WellKnownPath, #[case] identifier: &str) {
        let issuer = issuer("https://example.com/tenant/");
        let url = path.url(&issuer);
        assert_eq!(
            url.as_str(),
            format!("https://example.com/.well-known/{identifier}/tenant")
        );
    }

    #[rstest]
    #[case(WellKnownPath::CredentialIssuer, "openid-credential-issuer")]
    #[case(WellKnownPath::OauthAuthorizationServer, "oauth-authorization-server")]
    #[case(WellKnownPath::OpenidConfiguration, "openid-configuration")]
    fn test_well_known_url_with_multi_segment_paths(#[case] path: WellKnownPath, #[case] identifier: &str) {
        let issuer = issuer("https://example.com/tenant/sub-tenant");
        let url = path.url(&issuer);
        assert_eq!(
            url.as_str(),
            format!("https://example.com/.well-known/{identifier}/tenant/sub-tenant")
        );
    }
}
