use openid::Options;
use url::Url;

use super::Client;

const PARAM_CODE_CHALLENGE: &str = "code_challenge";
const PARAM_CODE_CHALLENGE_METHOD: &str = "code_challenge_method";

const CHALLENGE_METHOD_S256: &str = "S256";

impl Client {
    /// This wraps `openid::Client.auth_url()`, but adds a PKCE code challenge.
    pub fn auth_url_pkce(&self, options: &Options, code_challenge: &str) -> Url {
        let mut auth_url = self.0.auth_url(options);

        // Add PKCE challenge
        auth_url
            .query_pairs_mut()
            .append_pair(PARAM_CODE_CHALLENGE, code_challenge)
            .append_pair(PARAM_CODE_CHALLENGE_METHOD, CHALLENGE_METHOD_S256);

        auth_url
    }
}
