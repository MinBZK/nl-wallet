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
    pub fn parse_from_uri(uri: &Url) -> Result<Self, DisclosureUriError> {
        if uri.cannot_be_a_base() {
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

    use super::DisclosureUriError;
    use super::VpDisclosureUriData;

    #[rstest]
    #[case("scheme://host.name/some/path?foo=bar", "foo=bar")]
    #[case("scheme://host.name/some/path?key1=value1&key2=value2", "key1=value1&key2=value2")]
    fn test_parse_disclosure_uri(#[case] uri: Url, #[case] expected_query: &str) {
        let disclosure_uri = VpDisclosureUriData::parse_from_uri(&uri).expect("Could not parse disclosure URI");

        assert_eq!(disclosure_uri.query, expected_query);
    }

    #[test]
    fn test_parse_disclosure_uri_error_no_query() {
        let error = VpDisclosureUriData::parse_from_uri(&"scheme://host.name/some/path".parse().unwrap())
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Malformed(_));
    }
}
