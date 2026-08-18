use http_utils::reqwest::HttpClient;
use serde::de::DeserializeOwned;
use url::Url;

use crate::issuer_identifier::IssuerIdentifier;

pub trait WellKnownMetadata: DeserializeOwned {
    const PATH: &'static str;

    fn well_known_url(issuer: &IssuerIdentifier) -> Url {
        let url = issuer.as_base_url().as_ref();
        let path = url.path();
        let path = path.strip_suffix('/').unwrap_or(path);
        url.join(&format!("/.well-known/{}{path}", Self::PATH))
            .expect("both paths are already safe url encoded")
    }

    fn issuer_identifier(&self) -> &IssuerIdentifier;

    async fn fetch_well_known_json(client: &HttpClient, issuer: &IssuerIdentifier) -> Result<Self, WellKnownError> {
        let url = Self::well_known_url(issuer);
        let metadata: Self = client.get_json(url).await?;
        if metadata.issuer_identifier() != issuer {
            return Err(WellKnownError::IssuerIdentifierMismatch {
                expected: Box::new(issuer.clone()),
                received: Box::new(metadata.issuer_identifier().clone()),
            });
        }
        Ok(metadata)
    }
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::metadata::oauth_metadata::OidcProviderMetadata;

    fn issuer(s: &str) -> IssuerIdentifier {
        s.parse().unwrap()
    }

    #[rstest]
    #[case("https://example.com/", "")]
    #[case("https://example.com/tenant", "/tenant")]
    #[case("https://example.com/tenant/", "/tenant")]
    #[case("https://example.com/tenant/sub-tenant", "/tenant/sub-tenant")]
    fn test_well_known_url_oauth_authorization_server(#[case] issuer_url: &str, #[case] path_suffix: &str) {
        let issuer = issuer(issuer_url);
        let url = AuthorizationServerMetadata::well_known_url(&issuer);
        assert_eq!(
            url.as_str(),
            format!("https://example.com/.well-known/oauth-authorization-server{path_suffix}")
        );
    }

    #[rstest]
    #[case("https://example.com/", "")]
    #[case("https://example.com/tenant", "/tenant")]
    #[case("https://example.com/tenant/", "/tenant")]
    #[case("https://example.com/tenant/sub-tenant", "/tenant/sub-tenant")]
    fn test_well_known_url_openid_configuration(#[case] issuer_url: &str, #[case] path_suffix: &str) {
        let issuer = issuer(issuer_url);
        let url = OidcProviderMetadata::well_known_url(&issuer);
        assert_eq!(
            url.as_str(),
            format!("https://example.com/.well-known/openid-configuration{path_suffix}")
        );
    }
}
