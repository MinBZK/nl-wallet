use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use url::Url;

use crate::utils;

const PARAM_RETURN_URL: &str = "return_url";

#[derive(Debug, thiserror::Error)]
pub enum DisclosureUriError {
    #[error("URI is malformed: {0}")]
    Malformed(Url),
    #[error("could not decode reader engagement: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("could not parse return URL: {0}")]
    ReturnUrl(#[from] url::ParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(test, feature = "mock"))]
#[derive(Default)]
pub struct DisclosureUriData {
    pub reader_engagement_bytes: Vec<u8>,
    pub return_url: Option<Url>,
}

impl DisclosureUriData {
    /// Parse the `ReaderEngagement` bytes and a possible return URL from the disclosure URI.
    /// The `base_uri` argument is contained in the `Configuration`.
    pub fn parse_from_uri(uri: &Url, base_uri: &Url) -> Result<Self, DisclosureUriError> {
        // Check if both URIs can have path segments (see below) and
        // if the the base URI is actually a base of the disclosure URI.
        if uri.cannot_be_a_base() || base_uri.cannot_be_a_base() || !uri.as_str().starts_with(base_uri.as_str()) {
            return Err(DisclosureUriError::Malformed(uri.clone()));
        }

        // Get the number of path segments in the base URI, taking a trailing slash into account.
        let mut base_path_segment_count = base_uri.path_segments().map(|s| s.count()).unwrap_or_default();
        if base_uri.path().ends_with('/') {
            base_path_segment_count -= 1;
        }

        // Get the first path segment from the disclosure URI that is beyond
        // that of the base and check that it is not an empty string.
        let mut path_segments_iter = uri.path_segments().unwrap().skip(base_path_segment_count);
        let path_segment = path_segments_iter.next();
        let reader_engagement_base64 = path_segment.ok_or_else(|| DisclosureUriError::Malformed(uri.clone()))?;

        if reader_engagement_base64.is_empty() {
            return Err(DisclosureUriError::Malformed(uri.clone()));
        }

        // If there are additional path segments, consider that an error.
        if path_segments_iter.next().is_some() {
            return Err(DisclosureUriError::Malformed(uri.clone()));
        }

        // Decode the `ReaderEngagement` bytes from base64.
        let reader_engagement_bytes = URL_SAFE_NO_PAD.decode(reader_engagement_base64)?;

        // Parse an optional return URL from the query parameters.
        let return_url = utils::url::url_find_first_query_value(uri, PARAM_RETURN_URL)
            .map(|url| Url::parse(url.as_ref()))
            .transpose()?;

        let disclosure_uri = DisclosureUriData {
            reader_engagement_bytes,
            return_url,
        };

        Ok(disclosure_uri)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy",
        "scheme://host.name/some/path",
        b"foobar",
        None
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy",
        "scheme://host.name/some/path/",
        b"foobar",
        None
    )]
    #[case("scheme://host.name/Zm9vYmFy", "scheme://host.name", b"foobar", None)]
    #[case("scheme://host.name/Zm9vYmFy", "scheme://host.name/", b"foobar", None)]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com",
        "scheme://host.name/some/path",
        b"foobar",
        Some("https://example.com")
    )]
    fn test_parse_disclosure_uri(
        #[case] uri: Url,
        #[case] base_uri: Url,
        #[case] expected_bytes: &[u8],
        #[case] expected_return_url: Option<&str>,
    ) {
        let disclosure_uri =
            DisclosureUriData::parse_from_uri(&uri, &base_uri).expect("Could not parse disclosure URI");

        assert_eq!(disclosure_uri.reader_engagement_bytes, expected_bytes);
        assert_eq!(
            disclosure_uri.return_url,
            expected_return_url.map(|url| Url::parse(url).unwrap())
        );
    }

    #[rstest]
    #[case("httsp://example.com/Zm9vYmFy", "scheme://host.name")]
    #[case("scheme://host.name/some/path/Zm9vYmFy/blah", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path/")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path/")]
    fn test_parse_disclosure_uri_error_malformed(#[case] uri: Url, #[case] base_uri: Url) {
        let error = DisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Malformed(_));
    }

    #[rstest]
    #[case("scheme://host.name/some/path/foobar", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/Zm9vYmFyCg==", "scheme://host.name/some/path")]
    fn test_parse_disclosure_uri_error_base64(#[case] uri: Url, #[case] base_uri: Url) {
        let error = DisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Base64(_));
    }

    #[rstest]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=foobar",
        "scheme://host.name/some/path"
    )]
    fn test_parse_disclosure_uri_error_return_url(#[case] uri: Url, #[case] base_uri: Url) {
        let error = DisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::ReturnUrl(_));
    }
}
