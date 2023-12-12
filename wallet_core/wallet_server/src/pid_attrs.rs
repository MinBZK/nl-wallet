use futures::TryFutureExt;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use nl_wallet_mdoc::basic_sa_ext::UnsignedMdoc;
use reqwest::header::CONTENT_TYPE;

use openid4vc::token::{TokenErrorResponse, TokenRequest, TokenRequestGrantType, TokenResponse};

use crate::digid::{self, AttributesLookup, BsnLookup, OpenIdClient};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] reqwest::Error),
    #[error("error requesting token: {0:?}")]
    TokenRequest(TokenErrorResponse),
    #[error(transparent)]
    Digid(#[from] digid::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
}

pub async fn pid_attrs(
    token_request: TokenRequest,
    openid_client: &OpenIdClient,
    http_client: &reqwest::Client,
    attrs_lookup: &impl AttributesLookup,
) -> Result<Vec<UnsignedMdoc>, Error> {
    let code = match token_request.grant_type {
        TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } => pre_authorized_code,
        _ => panic!("must be a pre-authorized_code token request"),
    };

    let openid_token_request = serde_urlencoded::to_string(TokenRequest {
        grant_type: TokenRequestGrantType::AuthorizationCode { code },
        ..token_request
    })?;

    let openid_token_response: TokenResponse = http_client
        .post(openid_client.openid_client.config().token_endpoint.clone())
        .header(CONTENT_TYPE, APPLICATION_WWW_FORM_URLENCODED.as_ref())
        .body(dbg!(openid_token_request))
        .send()
        .map_err(Error::from)
        .and_then(|response| async {
            // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                let error = response.json::<TokenErrorResponse>().await?;
                Err(Error::TokenRequest(error))
            } else {
                let text = response.json().await?;
                Ok(text)
            }
        })
        .await?;

    let bsn = openid_client.bsn(&openid_token_response.access_token).await?;
    let unsigned_mdocs = attrs_lookup.attributes(&bsn);

    Ok(unsigned_mdocs)
}
