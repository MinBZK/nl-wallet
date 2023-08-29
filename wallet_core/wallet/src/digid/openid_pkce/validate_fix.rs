use chrono::Duration;
use openid::{
    error::{Error, Mismatch, Missing, Validation},
    validation::{validate_token_exp, validate_token_issuer, validate_token_nonce},
    Claims, IdToken, SingleOrMultiple, StandardClaims,
};

use super::Client;

/// This is a fix for `openid::validation::validate_token_aud()`.
///
/// The `openid` crate does not validate whether `SingleOrMultiple::Multiple` actually contains
/// multiple entries, so here we validate this according to the Robustness principle.
// TODO: submit PR for upstream inclusion in `openid` crate.
pub fn validate_token_aud<C: Claims>(claims: &C, client_id: &str) -> Result<(), Error> {
    if !claims.aud().contains(client_id) {
        return Err(Validation::Missing(Missing::Audience).into());
    }
    // By spec, if there are multiple auds, we must have an azp
    if let SingleOrMultiple::Multiple(azp_claim) = claims.aud() {
        // The next line is the fix:
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

impl Client {
    /// See the comment for `validate_token_aud` above.
    /// This is a direct copy of `openid::Client.validate_token()`.
    pub fn validate_token<'nonce, 'max_age>(
        &self,
        token: &IdToken<StandardClaims>,
        nonce: impl Into<Option<&'nonce str>>,
        max_age: impl Into<Option<&'max_age Duration>>,
    ) -> Result<(), Error> {
        let claims = token.payload()?;
        let config = self.0.config();

        validate_token_issuer(claims, config)?;

        validate_token_nonce(claims, nonce)?;

        validate_token_aud(claims, &self.0.client_id)?;

        validate_token_exp(claims, max_age)?;

        Ok(())
    }
}
