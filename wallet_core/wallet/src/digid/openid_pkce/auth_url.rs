use openid::Options;
use url::Url;

use crate::{pkce::PkcePair, utils::url::url_with_query_pairs};

use super::Client;

const PARAM_CODE_CHALLENGE: &str = "code_challenge";
const PARAM_CODE_CHALLENGE_METHOD: &str = "code_challenge_method";

impl Client {
    /// This wraps `openid::Client.auth_url()`, but adds a PKCE code challenge.
    pub fn auth_url<P>(&self, options: &Options, pkce_pair: &P) -> Url
    where
        P: PkcePair,
    {
        let url = url_with_query_pairs(
            self.0.auth_url(options),
            &[
                (PARAM_CODE_CHALLENGE, pkce_pair.code_challenge()),
                (PARAM_CODE_CHALLENGE_METHOD, P::CODE_CHALLENGE_METHOD),
            ],
        );

        url
    }
}

#[cfg(test)]
mod tests {
    use openid::Config;
    use serde_json::json;

    use crate::pkce::MockPkcePair;

    use super::*;

    #[test]
    fn test_auth_url_pkce() {
        // Abuse serde to create `Config`, since `Config` does not implement `Default`.
        let config = serde_json::from_value::<Config>(json!({
            "issuer": "http://example.com",
            "authorization_endpoint": "http://example.com/oauth2/auth",
            "token_endpoint": "http://example.com/oauth2/token",
            "jwks_uri": "http://example.com/.well-known/jwks.json",
            "response_types_supported": []
        }))
        .expect("Could not create openid::Config.");

        let http_client = reqwest::Client::new();
        let client = openid::Client::new(
            config.into(),
            "foo".to_string(),
            "bar".to_string(),
            "http://example-client.com/oauth2/callback".to_string(),
            http_client,
            None,
        );
        let client = Client(client);

        let options = Options {
            scope: Some("scope_a scope_b scope_c".to_string()),
            state: Some("csrftoken".to_string()),
            nonce: Some("thisisthenonce".to_string()),
            ..Default::default()
        };
        let pkce_pair = {
            let mut pkce_pair = MockPkcePair::new();

            pkce_pair
                .expect_code_challenge()
                .return_const("pkcecodechallenge".to_string());

            pkce_pair
        };

        let url = client.auth_url(&options, &pkce_pair);

        assert_eq!(
            url,
            Url::parse(
                "http://example.com/oauth2/auth?response_type=code&client_id=foo&redirect_uri=\
                 http%3A%2F%2Fexample-client.com%2Foauth2%2Fcallback&scope=openid+scope_a+scope_b+scope_c\
                 &state=csrftoken&nonce=thisisthenonce&code_challenge=pkcecodechallenge&code_challenge_method=INVALID"
            )
            .unwrap()
        );
    }
}
