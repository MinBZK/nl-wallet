use std::borrow::Cow;

use url::Url;

pub fn url_find_first_query_value<'a>(url: &'a Url, query_key: &str) -> Option<Cow<'a, str>> {
    url.query_pairs()
        .find(|(key, _)| key == query_key)
        .map(|(_, value)| value)
}

pub fn url_with_query_pairs(mut url: Url, query_pairs: &[(&str, &str)]) -> Url {
    if query_pairs.is_empty() {
        return url;
    }

    {
        let mut query = url.query_pairs_mut();

        query_pairs.iter().for_each(|(name, value)| {
            query.append_pair(name, value);
        });
    }

    url
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

    #[rstest]
    #[case("http://example.com", [], "http://example.com")]
    #[case("http://example.com", [("foo", "bar"), ("bleh", "blah")], "http://example.com?foo=bar&bleh=blah")]
    #[case("http://example.com", [("foo", ""), ("foo", "more_foo")], "http://example.com?foo=&foo=more_foo")]
    fn test_url_with_query_pairs<const N: usize>(
        #[case] url: Url,
        #[case] query_pairs: [(&str, &str); N],
        #[case] expected: Url,
    ) {
        let url = url_with_query_pairs(url, &query_pairs);

        assert_eq!(url, expected);
    }
}
