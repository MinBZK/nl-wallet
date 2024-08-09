use url::Url;

use error_category::ErrorCategory;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum DisclosureUriError {
    #[error("URI is malformed: {0}")]
    Malformed(Url),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpDisclosureUriData {
    pub query: String,
}

impl VpDisclosureUriData {
    pub fn parse_from_uri(uri: &Url, base_uri: &Url) -> Result<Self, DisclosureUriError> {
        // Check if both URIs can have path segments and if the the base URI is actually a base of the disclosure URI.
        if uri.cannot_be_a_base() || base_uri.cannot_be_a_base() || !uri.as_str().starts_with(base_uri.as_str()) {
            return Err(DisclosureUriError::Malformed(uri.clone()));
        }

        Ok(Self {
            query: uri
                .query()
                .ok_or(DisclosureUriError::Malformed(uri.clone()))?
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use rstest::rstest;
    use url::Url;

    use super::{DisclosureUriError, VpDisclosureUriData};

    #[rstest]
    #[case("scheme://host.name/some/path?foo=bar", "scheme://host.name/some/path", "foo=bar")]
    #[case(
        "scheme://host.name/some/path?key1=value1&key2=value2",
        "scheme://host.name/some/path",
        "key1=value1&key2=value2"
    )]
    fn test_parse_disclosure_uri(#[case] uri: Url, #[case] base_uri: Url, #[case] expected_query: &str) {
        let disclosure_uri =
            VpDisclosureUriData::parse_from_uri(&uri, &base_uri).expect("Could not parse disclosure URI");

        assert_eq!(disclosure_uri.query, expected_query);
    }

    #[rstest]
    #[case("https://example.com/foobar", "scheme://host.name")]
    #[case("scheme://host.name/some/path/foobar/blah", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path/")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path/")]
    #[case("scheme://host.name/some/path", "scheme://host.name")]
    fn test_parse_disclosure_uri_error_malformed(#[case] uri: Url, #[case] base_uri: Url) {
        let error = VpDisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Malformed(_));
    }
}
