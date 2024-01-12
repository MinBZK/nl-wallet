use base64::prelude::*;
use serde::Deserialize;
use url::Url;

use nl_wallet_mdoc::verifier::SessionType;

#[derive(Debug, thiserror::Error)]
pub enum DisclosureUriError {
    #[error("URI is malformed: {0}")]
    Malformed(Url),
    #[error("could not decode reader engagement: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("could not parse URL parameters: {0}")]
    InvalidParameters(#[from] serde_urlencoded::de::Error),
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "mock"), derive(Debug))]
pub struct DisclosureUriData {
    pub reader_engagement_bytes: Vec<u8>,
    pub return_url: Option<Url>,
    pub session_type: SessionType,
}

#[derive(Deserialize)]
struct DisclosureParams {
    pub return_url: Option<Url>,
    pub session_type: SessionType,
}

impl DisclosureUriData {
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

        // Parse an optional return URL and session type from the query parameters.
        let DisclosureParams {
            return_url,
            session_type,
        } = serde_urlencoded::from_str::<DisclosureParams>(uri.query().unwrap_or(""))
            .map_err(DisclosureUriError::InvalidParameters)?;

        let disclosure_uri = DisclosureUriData {
            reader_engagement_bytes,
            return_url,
            session_type,
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
        "scheme://host.name/some/path/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com&session_type=same_device",
        "scheme://host.name/some/path",
        b"foobar",
        Some(Url::parse("https://example.com").unwrap()),
        SessionType::SameDevice
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com&session_type=same_device&irrelevant_parameter=value",
        "scheme://host.name/some/path",
        b"foobar",
        Some(Url::parse("https://example.com").unwrap()),
        SessionType::SameDevice
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?session_type=same_device&return_url=https%3A%2F%2Fexample.com",
        "scheme://host.name/some/path",
        b"foobar",
        Some(Url::parse("https://example.com").unwrap()),
        SessionType::SameDevice
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=https%3A%2F%2Fexample.com&session_type=cross_device",
        "scheme://host.name/some/path",
        b"foobar",
        Some(Url::parse("https://example.com").unwrap()),
        SessionType::CrossDevice
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?session_type=same_device",
        "scheme://host.name/some/path",
        b"foobar",
        None,
        SessionType::SameDevice
    )]
    fn test_parse_disclosure_uri(
        #[case] uri: Url,
        #[case] base_uri: Url,
        #[case] expected_bytes: &[u8],
        #[case] expected_return_url: Option<Url>,
        #[case] expected_session_type: SessionType,
    ) {
        let disclosure_uri =
            DisclosureUriData::parse_from_uri(&uri, &base_uri).expect("Could not parse disclosure URI");

        assert_eq!(disclosure_uri.reader_engagement_bytes, expected_bytes);
        assert_eq!(disclosure_uri.session_type, expected_session_type);
        assert_eq!(disclosure_uri.return_url, expected_return_url);
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
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?session_type=not_a_session",
        "scheme://host.name/some/path"
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?return_url=&session_type=same_device",
        "scheme://host.name/some/path"
    )]
    #[case(
        "scheme://host.name/some/path/Zm9vYmFy?session_type=",
        "scheme://host.name/some/path"
    )]
    fn test_parse_disclosure_uri_error_return_url(#[case] uri: Url, #[case] base_uri: Url) {
        let error = DisclosureUriData::parse_from_uri(&uri, &base_uri)
            .expect_err("Parsing disclosure URI should have resulted in error");

        assert_matches!(error, DisclosureUriError::InvalidParameters(_));
    }
}
