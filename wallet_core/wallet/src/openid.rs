use async_trait::async_trait;
use chrono::Duration;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use openid::{
    biscuit::SingleOrMultiple,
    error::{ClientError, Error as OpenIdError, Mismatch, Missing, Validation},
    validation::{validate_token_exp, validate_token_issuer, validate_token_nonce},
    Bearer, Claims, Client, IdToken, OAuth2Error, Provider, StandardClaims,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;

const APPLICATION_JWT: &str = "application/jwt";

#[async_trait]
pub trait OpenIdClientExtensions {
    async fn invoke_token_endpoint(&self, body: String) -> Result<Bearer, ClientError>;
    fn validate_custom_token<'nonce, 'max_age>(
        &self,
        token: &IdToken<StandardClaims>,
        nonce: impl Into<Option<&'nonce str>>,
        max_age: impl Into<Option<&'max_age Duration>>,
    ) -> Result<(), OpenIdError>;
}

// The `openid` crate does not validate whether `SingleOrMultiple::Multiple` actually contains
// multiple entries, so here we validate this according to the Robustness principle.
// TODO submit PR for upstream inclusion in `openid` crate
fn validate_token_aud<C: Claims>(claims: &C, client_id: &str) -> Result<(), OpenIdError> {
    if !claims.aud().contains(client_id) {
        return Err(Validation::Missing(Missing::Audience).into());
    }
    // By spec, if there are multiple auds, we must have an azp
    if let SingleOrMultiple::Multiple(azp_claim) = claims.aud() {
        if azp_claim.len() > 1 && claims.azp().is_none() {
            return Err(Validation::Missing(Missing::AuthorizedParty).into());
        }
    }
    // If there is an authorized party, it must be our client_id
    if let Some(actual) = claims.azp() {
        if actual != client_id {
            let expected = client_id.to_string();
            let actual = actual.to_string();
            return Err(Validation::Mismatch(Mismatch::AuthorizedParty { expected, actual }).into());
        }
    }

    Ok(())
}

#[async_trait]
impl OpenIdClientExtensions for Client {
    async fn invoke_token_endpoint(&self, body: String) -> Result<Bearer, ClientError> {
        let json = self
            .http_client
            .post(self.provider.token_uri().clone())
            .header(ACCEPT, APPLICATION_JWT)
            .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let error: Result<OAuth2Error, _> = serde_json::from_value(json.clone());

        // TODO check for response state instead, and decode based on that
        if let Ok(error) = error {
            Err(ClientError::from(error))
        } else {
            Ok(serde_json::from_value(json)?)
        }
    }

    // See comment at `validate_token_aud`.
    fn validate_custom_token<'nonce, 'max_age>(
        &self,
        token: &IdToken<StandardClaims>,
        nonce: impl Into<Option<&'nonce str>>,
        max_age: impl Into<Option<&'max_age Duration>>,
    ) -> Result<(), OpenIdError> {
        let claims = token.payload()?;
        let config = self.config();

        validate_token_issuer(claims, config)?;
        validate_token_nonce(claims, nonce)?;
        validate_token_aud(claims, &self.client_id)?;
        validate_token_exp(claims, max_age)?;

        Ok(())
    }
}
