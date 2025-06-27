use std::fmt::Display;

use cfg_if::cfg_if;
use http::header::InvalidHeaderValue;
use http::HeaderValue;
use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use url::Url;

#[nutype(
    validate(predicate = |u| !u.cannot_be_a_base()),
    derive(Debug, Clone, TryFrom, FromStr, Display, AsRef, PartialEq, Eq, Hash, Serialize, Deserialize),
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
const DISCLOSURE_BASED_ISSUANCE_BASE_PATH: &str = "disclosure_based_issuance";

#[inline]
pub fn issuance_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
    universal_link_base.join_base_url(ISSUANCE_BASE_PATH)
}

#[inline]
pub fn disclosure_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
    universal_link_base.join_base_url(DISCLOSURE_BASE_PATH)
}

#[inline]
pub fn disclosure_based_issuance_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
    universal_link_base.join_base_url(DISCLOSURE_BASED_ISSUANCE_BASE_PATH)
}

#[nutype(validate(predicate = |u| u.scheme() == "https"), derive(Debug, Clone, TryFrom, FromStr, Display, PartialEq, Eq, Serialize, Deserialize))]
pub struct HttpsUri(Url);

#[nutype(validate(predicate = |u| Origin::is_valid(u)), derive(Debug, Clone, TryFrom, PartialEq, Eq, Deserialize))]
pub struct Origin(Url);

impl Origin {
    fn is_valid(u: &Url) -> bool {
        cfg_if! {
            if #[cfg(feature = "allow_insecure_url")] {
                let allowed_schemes = ["https", "http"];
            } else {
                let allowed_schemes = ["https"];
            }
        }

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
            str += &format!(":{port}");
        }
        HeaderValue::try_from(str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum CorsOrigin {
    #[serde(rename = "*")]
    Any,
    #[serde(untagged)]
    Origins(Vec<Origin>),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ConnectSource {
    #[serde(rename = "'none'")]
    None,
    #[serde(untagged)]
    List(Vec<SourceExpression>),
}

impl Display for ConnectSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectSource::None => f.write_str("'none'"),
            ConnectSource::List(list) => {
                write!(f, "{}", list.iter().join(" "))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum SourceExpression {
    #[serde(rename = "'self'")]
    SelfSource,
    #[serde(untagged)]
    SchemeSource(Scheme),
    #[serde(untagged)]
    HostSource(Url),
}

impl Display for SourceExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceExpression::SelfSource => f.write_str("'self'"),
            SourceExpression::SchemeSource(scheme) => write!(f, "{scheme}"),
            SourceExpression::HostSource(url) => write!(f, "{url}"),
        }
    }
}

#[nutype(
    derive(Clone, Debug, PartialEq, Eq, TryFrom, Deserialize, Display),
    validate(regex = r"^([a-z][a-z0-9+\-.]*):$")
)]
pub struct Scheme(String);

#[cfg(feature = "server")]
mod axum {
    use tower_http::cors::AllowOrigin;

    use super::CorsOrigin;

    impl From<CorsOrigin> for AllowOrigin {
        fn from(value: CorsOrigin) -> Self {
            match value {
                CorsOrigin::Origins(allow_origins) => AllowOrigin::list(
                    allow_origins
                        .into_iter()
                        .map(|url| {
                            url.try_into()
                                .expect("cross_origin base_url should be parseable to header value")
                        })
                        .collect::<Vec<_>>(),
                ),
                CorsOrigin::Any => AllowOrigin::any(),
            }
        }
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
    async fn base_url(#[case] value: &str, #[case] expected: Result<(), ()>) {
        // The `BaseUrlParseError` that `nutype` returns does not implement `PartialEq`
        assert_eq!(value.parse::<BaseUrl>().map(|_| ()).map_err(|_| ()), expected);
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

    fn origin_urls(urls: Vec<&'static str>) -> CorsOrigin {
        let cors_urls = urls
            .into_iter()
            .map(|url| Url::parse(url).unwrap().try_into().unwrap())
            .collect::<Vec<_>>();
        CorsOrigin::Origins(cors_urls)
    }

    #[rstest]
    #[case(r#""*""#, CorsOrigin::Any)]
    #[case(r#"[]"#, origin_urls(vec![]))]
    #[case(r#"["https://example.com"]"#, origin_urls(vec!["https://example.com"]))]
    #[case(
        r#"["https://example.com", "https://example.com:8443"]"#,
        origin_urls(vec!["https://example.com", "https://example.com:8443"]),
    )]
    fn deserialize_origin(#[case] input: &str, #[case] expected: CorsOrigin) {
        let actual: CorsOrigin = serde_json::from_str(input).expect("json");
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(r#"invalid"#)]
    #[case(r#"["data:image/jpeg;base64,/9j/4AAQSkZJRgABAgAAZABkAAD"]"#)]
    fn deserialize_origin_errors(#[case] input: &str) {
        let _ = serde_json::from_str::<CorsOrigin>(input).expect_err("invalid json");
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
    pub struct Test {
        connect_src: ConnectSource,
    }

    #[rstest]
    #[case(r#""'none'""#, ConnectSource::None)]
    #[case(r#"[]"#, ConnectSource::List(vec![]))]
    #[case(r#"["'self'"]"#, ConnectSource::List(vec![SourceExpression::SelfSource]))]
    #[case(r#"["'self'", "wss:", "https://example.com"]"#, ConnectSource::List(vec![
        SourceExpression::SelfSource,
        SourceExpression::SchemeSource("wss:".to_string().try_into().unwrap()),
        SourceExpression::HostSource(Url::parse("https://example.com").unwrap())])
    )]
    #[case(r#"["http://localhost:8080"]"#, ConnectSource::List(vec![
        SourceExpression::HostSource(Url::parse("http://localhost:8080").unwrap())])
    )]
    fn deserialize_connect_source(#[case] input: &str, #[case] expected: ConnectSource) {
        let actual: Test = toml::from_str(&format!("connect_src = {input}")).expect("toml");
        assert_eq!(actual.connect_src, expected);
    }

    #[rstest]
    #[case(r#"none"#)]
    #[case(r#"["self"]"#)]
    #[case(r#"["scheme/"]"#)]
    fn deserialize_connect_source_errors(#[case] input: &str) {
        let _ = toml::from_str::<Test>(&format!("connect_src = {input}")).expect_err("invalid toml");
    }
}
