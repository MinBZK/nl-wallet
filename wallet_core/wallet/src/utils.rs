use std::borrow::Cow;

use url::Url;

pub fn url_find_first_query_value<'a>(url: &'a Url, query_key: &str) -> Option<Cow<'a, str>> {
    url.query_pairs()
        .find(|(key, _)| key == query_key)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_find_first_query_value() {
        let input_and_expected = vec![
            ("http://example.com", "foo", None),
            ("http://example.com?foo=bar", "foo", Some("bar")),
            ("http://example.com?foo=blah&foo=bar", "foo", Some("blah")),
            ("http://example.com?space=with%20space", "space", Some("with space")),
        ];

        input_and_expected.iter().for_each(|(url, query_key, expected)| {
            let url = Url::parse(url).unwrap();
            let result = url_find_first_query_value(&url, query_key);

            assert_eq!(result.as_ref().map(|value| value.as_ref()), *expected);
        });
    }
}
