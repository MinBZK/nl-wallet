use chrono::Duration;
use http::header::{ACCEPT, CONTENT_TYPE};
use mime::APPLICATION_WWW_FORM_URLENCODED;
use openid::{
    error::{ClientError, Error},
    provider::Provider,
    Bearer, OAuth2Error, StandardClaims, Token,
};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use super::Client;

const PARAM_CODE_VERIFIER: &str = "code_verifier";

const APPLICATION_JWT: &str = "application/jwt";

impl Client {
    /// This copies `openid::Client.authenticate()` and ammends it
    /// by sending a PKCE verifier in the token request.
    pub async fn authenticate_pkce(
        &self,
        auth_code: &str,
        pkce_verifier: &str,
        nonce: impl Into<Option<&str>>,
        max_age: impl Into<Option<&Duration>>,
    ) -> Result<Token<StandardClaims>, Error> {
        let bearer = self
            .request_token_pkce(auth_code, pkce_verifier)
            .await
            .map_err(Error::from)?;
        let mut token: Token<StandardClaims> = bearer.into();
        if let Some(id_token) = token.id_token.as_mut() {
            self.0.decode_token(id_token)?;
            // Use our fixed validation:
            self.validate_token(id_token, nonce, max_age)?;
        }
        Ok(token)
    }

    /// This copies `openid::Client.request_token_pkce()` and
    /// ammends it by adding a PKCE verifier to the request body.
    pub async fn request_token_pkce(&self, code: &str, pkce_verifier: &str) -> Result<Bearer, ClientError> {
        // Ensure the non thread-safe `Serializer` is not kept across
        // an `await` boundary by localizing it to this inner scope.
        let body = {
            let mut body = Serializer::new(String::new());
            body.append_pair("grant_type", "authorization_code");
            body.append_pair("code", code);

            if let Some(ref redirect_uri) = self.0.redirect_uri {
                body.append_pair("redirect_uri", redirect_uri);
            }

            body.append_pair("client_id", &self.0.client_id);
            body.append_pair(PARAM_CODE_VERIFIER, pkce_verifier);

            body.finish()
        };

        let json = self.post_token_jwt(body).await?;
        let token: Bearer = serde_json::from_value(json)?;
        Ok(token)
    }

    /// This copies `openid::Client.post_token_jwt()` and changes it to
    /// not use basic auth and receive JSON of content type "application/jwt".
    async fn post_token_jwt(&self, body: String) -> Result<Value, ClientError> {
        let json = self
            .0
            .http_client
            .post(self.0.provider.token_uri().clone())
            .header(ACCEPT, APPLICATION_JWT)
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let error: Result<OAuth2Error, _> = serde_json::from_value(json.clone());

        if let Ok(error) = error {
            Err(ClientError::from(error))
        } else {
            Ok(json)
        }
    }
}
