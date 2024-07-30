use base64::prelude::*;
use url::Url;

use error_category::ErrorCategory;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum DisclosureUriError {
    #[error("URI is malformed: {0}")]
    Malformed(Url),
    #[error("could not decode reader engagement: {0}")]
    Base64(#[from] base64::DecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpDisclosureUriData {
    pub query: String,
}

impl VpDisclosureUriData {
    pub fn parse_from_uri(uri: &Url, base_uri: &Url) -> Result<Self, DisclosureUriError> {
        // Check if the base URI is actually a base of the disclosure URI.
        if !uri.as_str().starts_with(base_uri.as_str()) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsoDisclosureUriData {
    pub reader_engagement_bytes: Vec<u8>,
}

impl IsoDisclosureUriData {
    /// Parse the `ReaderEngagement` bytes, a possible return URL and the session_type from the disclosure URI.
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
        let reader_engagement_bytes = BASE64_URL_SAFE_NO_PAD.decode(reader_engagement_base64)?;

        let disclosure_uri = IsoDisclosureUriData {
            reader_engagement_bytes,
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
    #[case("scheme://host.name/some/path/Zm9vYmFy", "scheme://host.name/some/path", b"foobar")]
    #[case(
        "scheme://host.name/some/path/cmVhZGVyLWVuZ2FnZW1lbnQtYnl0ZXM",
        "scheme://host.name/some/path",
        b"reader-engagement-bytes"
    )]
    #[case(
        "scheme://host.name/some/path/MTIzNDU2Nzg5MA",
        "scheme://host.name/some/path",
        b"1234567890"
    )]
    fn test_parse_disclosure_uri(#[case] uri: Url, #[case] base_uri: Url, #[case] expected_bytes: &[u8]) {
        let disclosure_uri =
            IsoDisclosureUriData::parse_from_uri(&uri, &base_uri).expect("Could not parse disclosure URI");

        assert_eq!(disclosure_uri.reader_engagement_bytes, expected_bytes);
    }

    #[rstest]
    #[case("httsp://example.com/Zm9vYmFy", "scheme://host.name")]
    #[case("scheme://host.name/some/path/Zm9vYmFy/blah", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path", "scheme://host.name/some/path/")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/", "scheme://host.name/some/path/")]
    fn test_parse_disclosure_uri_error_malformed(#[case] uri: Url, #[case] base_uri: Url) {
        let error = IsoDisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Malformed(_));
    }

    #[rstest]
    #[case("scheme://host.name/some/path/foobar", "scheme://host.name/some/path")]
    #[case("scheme://host.name/some/path/Zm9vYmFyCg==", "scheme://host.name/some/path")]
    fn test_parse_disclosure_uri_error_base64(#[case] uri: Url, #[case] base_uri: Url) {
        let error = IsoDisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::Base64(_));
    }
}
