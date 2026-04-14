use std::str::FromStr;

use derive_more::AsRef;
use derive_more::Display;
use derive_more::Eq;
use derive_more::PartialEq;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use url::Url;

use http_utils::urls::ALLOWED_HTTP_SCHEMES;
use http_utils::urls::BaseUrl;
use http_utils::urls::BaseUrlParseError;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum IssuerUrlError {
    #[error("issuer URL is not valid: {0}")]
    UrlParsing(#[from] BaseUrlParseError),

    #[error("issuer URL scheme is not \"https\": {0}")]
    SchemeNotHttps(Box<BaseUrl>),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum IssuerIdentifierError {
    #[error("issuer identifier is not a URL: {0}")]
    UrlParsing(#[from] IssuerUrlError),

    #[error("issuer identifier URL has query component: {0}")]
    HasQuery(Box<BaseUrl>),

    #[error("issuer identifier URL has fragment component: {0}")]
    HasFragment(Box<BaseUrl>),
}

#[derive(Debug, Clone, PartialEq, Eq, AsRef, Display, SerializeDisplay, DeserializeFromStr)]
pub struct IssuerUrl(BaseUrl);

/// A URL that uses the "https" scheme, as contained within the Credential Issuer Metadata.
impl IssuerUrl {
    pub fn try_new(url_str: &str) -> Result<Self, IssuerUrlError> {
        let url = url_str.parse::<BaseUrl>()?;

        // TODO (PVW-5612): Only allow HTTPS in local development environment.
        if !ALLOWED_HTTP_SCHEMES.contains(&url.as_ref().scheme()) {
            return Err(IssuerUrlError::SchemeNotHttps(Box::new(url)));
        }

        Ok(Self(url))
    }

    pub fn into_inner(self) -> BaseUrl {
        let Self(base_url) = self;

        base_url
    }

    pub fn as_url(&self) -> &Url {
        let Self(base_url) = self;

        base_url.as_ref()
    }

    pub fn into_url(self) -> Url {
        let Self(base_url) = self;

        base_url.into_inner()
    }

    pub fn join_issuer_url(&self, path: &str) -> Self {
        let base_url = self.as_ref().join_base_url(path);

        // The sheme cannot have changed, so the guarantees of the validation in `try_new()` still hold.
        Self(base_url)
    }
}

impl FromStr for IssuerUrl {
    type Err = IssuerUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

/// A (Credential) Issuer Identifier, as defined by
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-12.2.1> and
/// <https://www.rfc-editor.org/rfc/rfc8414.html#section-2>.
///
/// This wraps a URL with the following restrictions:
/// - The scheme should be "https".
/// - There should be no query component.
/// - There should be no fragement component.
///
/// Internally, this URL is represented both by a [`String`] and a [`IssuerUrl`] which
/// enables comparisons of the original string representation before URL normalization.
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Display, SerializeDisplay, DeserializeFromStr)]
#[display("{identifier}")]
pub struct IssuerIdentifier {
    #[as_ref(str)]
    identifier: String,
    #[partial_eq(skip)]
    #[serde_with(skip)]
    url: IssuerUrl,
}

impl IssuerIdentifier {
    pub fn try_new(identifier: String) -> Result<Self, IssuerIdentifierError> {
        let url = identifier.parse::<IssuerUrl>()?;

        if url.as_ref().as_ref().query().is_some() {
            return Err(IssuerIdentifierError::HasQuery(Box::new(url.into_inner())));
        }

        if url.as_ref().as_ref().fragment().is_some() {
            return Err(IssuerIdentifierError::HasFragment(Box::new(url.into_inner())));
        }

        Ok(Self { identifier, url })
    }

    pub fn as_base_url(&self) -> &BaseUrl {
        self.url.as_ref()
    }

    pub fn join_issuer_url(&self, path: &str) -> IssuerUrl {
        self.url.join_issuer_url(path)
    }
}

impl FromStr for IssuerIdentifier {
    type Err = IssuerIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::IssuerIdentifier;
    use super::IssuerIdentifierErrorDiscriminants;
    use super::IssuerUrl;
    use super::IssuerUrlErrorDiscriminants;

    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case::ok("https://example.com/", Ok(()))]
    #[case::err_not_url("example.com", Err(IssuerUrlErrorDiscriminants::UrlParsing))]
    #[case::err_url_cannot_be_a_base("mailto:foo@bar.com", Err(IssuerUrlErrorDiscriminants::UrlParsing))]
    #[cfg_attr(
        feature = "allow_insecure_url",
        case::ok_http("http://example.com/", Ok(()))
    )]
    #[cfg_attr(
        not(feature = "allow_insecure_url"),
        case::err_http("http://example.com/", Err(IssuerUrlErrorDiscriminants::SchemeNotHttps))
    )]
    fn test_issuer_url_parsing(#[case] input: &str, #[case] expected_result: Result<(), IssuerUrlErrorDiscriminants>) {
        let parsed_result = input.parse::<IssuerUrl>();
        let deserialized_result = serde_json::from_value::<IssuerUrl>(json!(input));

        match expected_result {
            Ok(()) => {
                let parsed_issuer_url = parsed_result.expect("parsing IssuerUrl should succeed");
                let deserialized_issuer_url = deserialized_result.expect("deserializing IssuerUrl should succeed");

                assert_eq!(parsed_issuer_url, deserialized_issuer_url);
            }
            Err(expected_error) => {
                let parsed_error = parsed_result.expect_err("parsing IssuerUrl should fail");
                let deserialize_error = deserialized_result.expect_err("deserializing IssuerUrl should fail");

                assert_eq!(IssuerUrlErrorDiscriminants::from(&parsed_error), expected_error);
                assert!(deserialize_error.to_string().contains(&parsed_error.to_string()));
            }
        }
    }

    #[rstest]
    #[case::ok_without_path_without_slash("https://example.com", Ok("https://example.com/"))]
    #[case::ok_without_path_with_slash("https://example.com/", Ok("https://example.com/"))]
    #[case::ok_with_path_without_slash("https://example.com/this/path", Ok("https://example.com/this/path"))]
    #[case::ok_with_path_with_slash("https://example.com/this/path/", Ok("https://example.com/this/path/"))]
    #[case::err_not_url("example.com", Err(IssuerIdentifierErrorDiscriminants::UrlParsing))]
    #[case::err_url_cannot_be_a_base("mailto:foo@bar.com", Err(IssuerIdentifierErrorDiscriminants::UrlParsing))]
    #[cfg_attr(
        feature = "allow_insecure_url",
        case::ok_http("http://example.com/", Ok("http://example.com/"))
    )]
    #[cfg_attr(
        not(feature = "allow_insecure_url"),
        case::err_http("http://example.com/", Err(IssuerIdentifierErrorDiscriminants::UrlParsing))
    )]
    #[case::err_short_query("https://example.com/path?", Err(IssuerIdentifierErrorDiscriminants::HasQuery))]
    #[case::err_long_query(
        "https://example.com/?foo=bar&bleh=blah",
        Err(IssuerIdentifierErrorDiscriminants::HasQuery)
    )]
    #[case::err_fragment(
        "https://example.com/path#fragment",
        Err(IssuerIdentifierErrorDiscriminants::HasFragment)
    )]
    fn test_issuer_identifier_try_new(
        #[case] input: &str,
        #[case] expected_result: Result<&str, IssuerIdentifierErrorDiscriminants>,
    ) {
        let parsed_result = input.parse::<IssuerIdentifier>();
        let deserialized_result = serde_json::from_value::<IssuerIdentifier>(json!(input));

        match expected_result {
            Ok(expected_url) => {
                let parsed_identifier = parsed_result.expect("parsing IssuerIdentifier should succeed");
                let deserialized_identifier =
                    deserialized_result.expect("deserializing IssuerIdentifier should succeed");

                assert_eq!(parsed_identifier.as_base_url().as_ref().as_str(), expected_url);
                assert_eq!(parsed_identifier.as_ref(), input);
                assert_eq!(parsed_identifier, deserialized_identifier);
            }
            Err(expected_error) => {
                let parsed_error = parsed_result.expect_err("parsing IssuerIdentifier should fail");
                let deserialize_error = deserialized_result.expect_err("deserializing IssuerIdentifier should fail");

                assert_eq!(IssuerIdentifierErrorDiscriminants::from(&parsed_error), expected_error);
                assert!(deserialize_error.to_string().contains(&parsed_error.to_string()));
            }
        }
    }

    #[test]
    fn test_issuer_identifier_serialization() {
        let identifier = IssuerIdentifier::try_new("https://example.com".to_string()).unwrap();

        let json = serde_json::to_string(&identifier).expect("serialization to JSON should succeed");
        let deserialized_identifier =
            serde_json::from_str::<IssuerIdentifier>(&json).expect("deserialization from JSON should succeed");

        assert_eq!(deserialized_identifier, identifier);
    }
}
