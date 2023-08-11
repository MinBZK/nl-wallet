use std::borrow::Cow;

use url::Url;

pub fn url_find_first_query_value<'a>(url: &'a Url, query_key: &str) -> Option<Cow<'a, str>> {
    url.query_pairs()
        .find(|(key, _)| key == query_key)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("http://example.com", "foo", None)]
    #[case("http://example.com?foo=bar", "foo", Some("bar"))]
    #[case("http://example.com?foo=blah&foo=bar", "foo", Some("blah"))]
    #[case("http://example.com?space=with%20space", "space", Some("with space"))]
    fn test_url_find_first_query_value(#[case] url: Url, #[case] key: &str, #[case] expected: Option<&str>) {
        let result = url_find_first_query_value(&url, key);

        assert_eq!(result.as_ref().map(|value| value.as_ref()), expected);
    }
}
