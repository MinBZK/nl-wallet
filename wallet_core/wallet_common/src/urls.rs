use http::{header::InvalidHeaderValue, HeaderValue};
use nutype::nutype;
use url::Url;

#[nutype(
    validate(predicate = |u| !u.cannot_be_a_base()),
    derive(FromStr, Debug, Clone, Deserialize, Serialize, Display, AsRef, TryFrom, PartialEq, Eq, Hash),
)]
pub struct BaseUrl(Url);

impl BaseUrl {
    // removes leading forward slashes, calls `Url::join` and unwraps the result
    // the idea behind this is that a BaseURL is intended to be joined with a relative path and not an absolute path
    pub fn join(&self, input: &str) -> Url {
        let mut ret = self.as_ref().clone();
        // both safe to unwrap because we know the URL is a valid base URL
        if !ret.path().ends_with('/') {
            ret.path_segments_mut().unwrap().push("/");
        }
        ret.join(input.trim_start_matches('/')).unwrap()
    }

    // call .join, but converted into a BaseUrl
    pub fn join_base_url(&self, input: &str) -> Self {
        self.join(input).try_into().unwrap()
    }
}

pub const DEFAULT_UNIVERSAL_LINK_BASE: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/";
const ISSUANCE_BASE_PATH: &str = "return-from-digid";
const DISCLOSURE_BASE_PATH: &str = "disclosure";

#[inline]
pub fn issuance_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
    universal_link_base.join_base_url(ISSUANCE_BASE_PATH)
}

#[inline]
pub fn disclosure_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
    universal_link_base.join_base_url(DISCLOSURE_BASE_PATH)
}

#[nutype(validate(predicate = |u| Origin::is_valid(u)), derive(TryFrom, Deserialize, Clone))]
pub struct Origin(Url);

impl Origin {
    fn is_valid(u: &Url) -> bool {
        #[cfg(feature = "allow_http_return_url")]
        let allowed_schemes = ["https", "http"];
        #[cfg(not(feature = "allow_http_return_url"))]
        let allowed_schemes = ["https"];

        (allowed_schemes.contains(&u.scheme()))
            && u.has_host()
            && u.fragment().is_none()
            && u.query().is_none()
            && u.path() == "/"
        // trailing slash is stripped of when converting to `HeaderValue`
    }
}

impl TryFrom<Origin> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: Origin) -> Result<Self, Self::Error> {
        let url = value.into_inner();
        let mut str = format!("{0}://{1}", url.scheme(), url.host_str().unwrap(),);
        if let Some(port) = url.port() {
            str += &format!(":{0}", port);
        }
        HeaderValue::try_from(str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("https://example.com/", Ok(()))]
    #[case("https://example.com/", Ok(()))]
    #[case("https://example.com/path/", Ok(()))]
    #[case("https://example.com/path", Ok(()))] // this is okay, since the `.join` method will add a trailing slash
    #[case("data:image/jpeg;base64,/9j/4AAQSkZJRgABAgAAZABkAAD", Err(()))]
    #[tokio::test]
    async fn base_url(#[case] value: &str, #[case] expected_err: Result<(), ()>) {
        // The `BaseUrlParseError` that `nutype` returns does not implement `PartialEq`
        assert_eq!(value.parse::<BaseUrl>().map(|_| ()).map_err(|_| ()), expected_err);
    }

    #[rstest]
    #[case("https://example.com/", "to", "https://example.com/to")]
    #[case("https://example.com/", "/to", "https://example.com/to")]
    #[case("https://example.com/", "to/", "https://example.com/to/")]
    #[case("https://example.com/", "/to/", "https://example.com/to/")]
    #[case("https://example.com/", "path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "/path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/", "/path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "to/success/", "https://example.com/path/to/success/")]
    // if path is absolute, remove leading '/'
    #[case("https://example.com/path/", "/to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "/to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "/to/success/", "https://example.com/path/to/success/")]
    #[tokio::test]
    async fn base_url_join(#[case] value: BaseUrl, #[case] path: &str, #[case] expected: &str) {
        assert_eq!(value.join(path).as_str(), expected);
        assert_eq!(value.join_base_url(path).as_ref().as_str(), expected);
    }
}
