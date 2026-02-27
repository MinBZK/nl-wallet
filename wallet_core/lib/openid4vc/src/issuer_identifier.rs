use std::str::FromStr;

use derive_more::Display;
use derive_more::Eq;
use derive_more::Into;
use derive_more::PartialEq;
use serde::Deserialize;
use serde::Serialize;

use http_utils::urls::ALLOWED_HTTP_SCHEMES;
use http_utils::urls::BaseUrl;
use http_utils::urls::BaseUrlParseError;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum CredentialIssuerIdentifierError {
    #[error("credential issuer identifier is not a URL: {0}")]
    UrlParsing(#[from] BaseUrlParseError),
    #[error("credential issuer identifier URL scheme is not \"https\": {0}")]
    SchemeNotHttps(BaseUrl),
    #[error("credential issuer identifier URL has query component: {0}")]
    HasQuery(BaseUrl),
    #[error("credential issuer identifier URL has fragment component: {0}")]
    HasFragment(BaseUrl),
}

/// A Credential Issuer Identifier, as defined by
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-12.2.1>.
///
/// This wraps a URL with the following restrictions:
/// - The scheme should be "https".
/// - There should be no query component.
/// - There should be no fragement component.
///
/// Internally, this URL is represented both by a [`String`] and a [`BaseUrl`] which
/// enables comparisons of the original string representation before URL normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Into, Display)]
#[display("{identifier}")]
#[serde(try_from = "String", into = "String")]
pub struct CredentialIssuerIdentifier {
    identifier: String,
    #[partial_eq(skip)]
    #[serde(skip)]
    #[into(skip)]
    url: BaseUrl,
}

impl CredentialIssuerIdentifier {
    pub fn try_new(identifier: String) -> Result<Self, CredentialIssuerIdentifierError> {
        let url = identifier.parse::<BaseUrl>()?;

        // TODO (PVW-5612): Only allow HTTPS in local development environment.
        if !ALLOWED_HTTP_SCHEMES.contains(&url.as_ref().scheme()) {
            return Err(CredentialIssuerIdentifierError::SchemeNotHttps(url));
        }

        if url.as_ref().query().is_some() {
            return Err(CredentialIssuerIdentifierError::HasQuery(url));
        }

        if url.as_ref().fragment().is_some() {
            return Err(CredentialIssuerIdentifierError::HasFragment(url));
        }

        Ok(Self { identifier, url })
    }

    pub fn as_base_url(&self) -> &BaseUrl {
        &self.url
    }
}

impl TryFrom<String> for CredentialIssuerIdentifier {
    type Error = CredentialIssuerIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl FromStr for CredentialIssuerIdentifier {
    type Err = CredentialIssuerIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s.to_string())
    }
}

impl AsRef<str> for CredentialIssuerIdentifier {
    fn as_ref(&self) -> &str {
        &self.identifier
    }
}

#[cfg(test)]
mod tests {
    use super::CredentialIssuerIdentifier;
    use super::CredentialIssuerIdentifierErrorDiscriminants;

    use rstest::rstest;

    #[rstest]
    #[case::ok_without_path_without_slash("https://example.com", Ok("https://example.com/"))]
    #[case::ok_without_path_with_slash("https://example.com/", Ok("https://example.com/"))]
    #[case::ok_with_path_without_slash("https://example.com/this/path", Ok("https://example.com/this/path"))]
    #[case::ok_with_path_with_slash("https://example.com/this/path/", Ok("https://example.com/this/path/"))]
    #[case::err_not_url("example.com", Err(CredentialIssuerIdentifierErrorDiscriminants::UrlParsing))]
    #[case::err_url_cannot_be_a_base(
        "mailto:foo@bar.com",
        Err(CredentialIssuerIdentifierErrorDiscriminants::UrlParsing)
    )]
    #[cfg_attr(
        feature = "allow_insecure_url",
        case::ok_http("http://example.com/", Ok("http://example.com/"))
    )]
    #[cfg_attr(
        not(feature = "allow_insecure_url"),
        case::err_http(
            "http://example.com/",
            Err(CredentialIssuerIdentifierErrorDiscriminants::SchemeNotHttps)
        )
    )]
    #[case::err_short_query(
        "https://example.com/path?",
        Err(CredentialIssuerIdentifierErrorDiscriminants::HasQuery)
    )]
    #[case::err_long_query(
        "https://example.com/?foo=bar&bleh=blah",
        Err(CredentialIssuerIdentifierErrorDiscriminants::HasQuery)
    )]
    #[case::err_fragment(
        "https://example.com/path#fragment",
        Err(CredentialIssuerIdentifierErrorDiscriminants::HasFragment)
    )]
    fn test_credential_issuer_identifier_try_new(
        #[case] input: &str,
        #[case] expected_result: Result<&str, CredentialIssuerIdentifierErrorDiscriminants>,
    ) {
        let result = CredentialIssuerIdentifier::try_new(input.to_string());

        match expected_result {
            Ok(expected_url) => {
                let identifier = result.expect("CredentialIssuerIdentifier try_new() should succeed");

                assert_eq!(identifier.as_base_url().as_ref().as_str(), expected_url);
                assert_eq!(identifier.as_ref(), input);
            }
            Err(expected_error) => {
                let error = result.expect_err("CredentialIssuerIdentifier try_new() should fail");

                assert_eq!(
                    CredentialIssuerIdentifierErrorDiscriminants::from(error),
                    expected_error
                );
            }
        }
    }

    #[test]
    fn test_credential_issuer_identifier_serialization() {
        let identifier = CredentialIssuerIdentifier::try_new("https://example.com".to_string()).unwrap();

        let json = serde_json::to_string(&identifier).expect("serialization to JSON should succeed");
        let deserialized_identifier = serde_json::from_str::<CredentialIssuerIdentifier>(&json)
            .expect("deserialization from JSON should succeed");

        assert_eq!(deserialized_identifier, identifier);
    }
}
