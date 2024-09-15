use cfg_if::cfg_if;
use nutype::nutype;
use strfmt::strfmt;

use url::Url;

use crate::server_state::SessionToken;

#[nutype(
    derive(Debug, Clone, FromStr, Serialize, Deserialize),
    validate(predicate = ReturnUrlTemplate::is_valid_return_url_template),
)]
pub struct ReturnUrlTemplate(String);

impl ReturnUrlTemplate {
    pub fn into_url(self, session_token: &SessionToken) -> Url {
        strfmt!(&self.into_inner(), session_token => session_token.to_string())
            .expect("valid ReturnUrlTemplate should always format")
            .parse()
            .expect("formatted ReturnUrlTemplate should always be a valid URL")
    }

    fn is_valid_return_url_template(s: &str) -> bool {
        cfg_if! {
            if #[cfg(feature = "allow_http_return_url")] {
                const ALLOWED_SCHEMES: [&str; 2] = ["https", "http"];
            } else {
                const ALLOWED_SCHEMES: [&str; 1] = ["https"];
            }
        }

        // It should be a valid URL when removing the template parameter.
        let s = s.replace("{session_token}", "");
        let url = s.parse::<Url>();

        url.is_ok_and(|url| ALLOWED_SCHEMES.contains(&url.scheme()))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::ReturnUrlTemplate;

    #[rstest]
    #[case("https://example.com/{session_token}", true)]
    #[case("https://example.com/return/{session_token}", true)]
    #[case("https://example.com/return/{session_token}/url", true)]
    #[case("https://example.com/{session_token}/", true)]
    #[case("https://example.com/return/{session_token}/", true)]
    #[case("https://example.com/return/{session_token}/url/", true)]
    #[case("https://example.com/return/{session_token}?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_token}/{session_token}", true)]
    #[case("https://example.com/", true)]
    #[case("https://example.com/return", true)]
    #[case("https://example.com/return/url", true)]
    #[case("https://example.com/return/", true)]
    #[case("https://example.com/return/url/", true)]
    #[case("https://example.com/return/?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_token}/{not_session_token}", true)]
    #[case("file://etc/passwd", false)]
    #[case("file://etc/{session_token}", false)]
    #[case("https://{session_token}", false)]
    #[cfg_attr(feature = "allow_http_return_url", case("http://example.com/{session_token}", true))]
    #[cfg_attr(
        not(feature = "allow_http_return_url"),
        case("http://example.com/{session_token}", false)
    )]
    fn test_return_url_template(#[case] return_url_string: String, #[case] should_parse: bool) {
        assert_eq!(return_url_string.parse::<ReturnUrlTemplate>().is_ok(), should_parse);
        assert_eq!(
            ReturnUrlTemplate::is_valid_return_url_template(&return_url_string),
            should_parse
        );
    }
}
