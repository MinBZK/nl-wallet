use std::{collections::HashMap, time::Duration};

use futures::future::TryFutureExt;
use http::header;
use josekit::{
    jwe::{self, alg::rsaes::RsaesJweDecrypter},
    JoseError,
};
use openid::{
    biscuit::{
        errors as biscuit_errors, jwa::SignatureAlgorithm, ClaimsSet, CompactJson, CompactPart, ValidationOptions, JWT,
    },
    error as openid_errors, Empty, OAuth2Error,
};
use serde_json::Value;
use tracing::debug;
use url::Url;

use crate::{app::BsnLookup, settings};

const APPLICATION_JWT: &str = "application/jwt";
const BSN_KEY: &str = "bsn";

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

// TODO: Replace with deserializable struct once we're certain about the claim(s).
pub type AttributeMap = HashMap<String, Value>;
pub type UserInfoJWT = JWT<AttributeMap, Empty>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    OpenId(#[from] openid_errors::Error),
    #[error(transparent)]
    OpenIdClient(#[from] openid_errors::ClientError),
    #[error(transparent)]
    OpenIdUserinfo(#[from] openid_errors::Userinfo),
    #[error(transparent)]
    JoseKit(#[from] JoseError),
    #[error("no BSN found in response from OIDC server")]
    NoBSN,
    #[error(transparent)]
    Jwe(#[from] biscuit_errors::Error),
    #[error(transparent)]
    JweValidation(#[from] biscuit_errors::ValidationError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

/// An OIDC client for exchanging an access token provided by the user for their BSN at the IdP.
pub struct OpenIdClient {
    client_id: String,
    issuer_url: Url,
    decrypter_private_key: RsaesJweDecrypter,
}

impl BsnLookup for OpenIdClient {
    async fn bsn(&self, access_token: &str) -> Result<String> {
        let userinfo_claims: UserInfoJWT = self
            .request_userinfo_decrypted_claims(access_token, &self.decrypter_private_key)
            .await?;

        OpenIdClient::bsn_from_claims(&userinfo_claims)?.ok_or(Error::NoBSN)
    }
}

impl OpenIdClient {
    pub async fn new(digid_settings: &settings::Digid) -> Result<Self> {
        let userinfo_client = OpenIdClient {
            client_id: digid_settings.client_id.clone(),
            issuer_url: digid_settings.issuer_url.clone(),
            decrypter_private_key: OpenIdClient::decrypter(&digid_settings.bsn_privkey)?,
        };
        Ok(userinfo_client)
    }

    async fn discover_client(&self) -> Result<openid::Client> {
        let http_client = reqwest::Client::builder();
        #[cfg(feature = "disable_tls_validation")]
        let http_client = http_client.danger_accept_invalid_certs(true);
        let http_client = http_client
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");
        let client = openid::Client::discover_with_client(
            http_client,
            self.client_id.clone(),
            None,
            None,
            self.issuer_url.clone(),
        )
        .await?;

        // Check that the userinfo endpoint was found by discovery
        _ = client
            .config()
            .userinfo_endpoint
            .as_ref()
            .ok_or(openid_errors::Userinfo::NoUrl)?;

        Ok(client)
    }

    pub fn decrypter(jwk_json: &str) -> Result<RsaesJweDecrypter> {
        let jwk = serde_json::from_str(jwk_json)?;
        let decrypter = jwe::RSA_OAEP.decrypter_from_jwk(&jwk)?;

        Ok(decrypter)
    }

    pub fn bsn_from_claims(userinfo_token: &UserInfoJWT) -> Result<Option<String>> {
        debug!("Processing userinfo claims");

        let userinfo_payload = userinfo_token.payload()?;
        userinfo_payload.registered.validate(ValidationOptions::default())?;

        debug!("Received userinfo claims are valid, extracting BSN");

        let bsn = userinfo_payload
            .private
            .get(BSN_KEY)
            .and_then(|value| value.as_str())
            .map(|s| s.to_string());

        Ok(bsn)
    }

    pub async fn request_userinfo_decrypted_claims<C, H>(
        &self,
        access_token: impl AsRef<str>,
        decrypter: &RsaesJweDecrypter,
    ) -> Result<JWT<C, H>>
    where
        ClaimsSet<C>: CompactPart,
        H: CompactJson,
    {
        debug!("Access token received, requesting user info from DigiD connector...");

        let client = self.discover_client().await?;

        // The JWK set should always be populated by discovery.
        let jwks = client
            .jwks
            .as_ref()
            .expect("OpenID client JWK set not populated by disovery");

        // Get userinfo endpoint from discovery, throw an error otherwise.
        let endpoint = client
            .config()
            .userinfo_endpoint
            .as_ref()
            .cloned()
            .expect("OpenID userinfo endpoint not populated by disovery");

        // Use the access_token to retrieve the userinfo as a JWE token.
        let jwe_token = client
            .http_client
            .post(endpoint)
            .header(header::ACCEPT, APPLICATION_JWT)
            .bearer_auth(access_token.as_ref())
            .send()
            .map_err(openid_errors::ClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the OAuth2Error JSON
                // body and return an error. Otherwise just retrieve the body as text.
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<OAuth2Error>().await?;

                    Err(openid_errors::ClientError::from(error))
                } else {
                    let text = response.text().await?;

                    Ok(text)
                }
            })
            .await?;

        debug!("User info retreived, decrypting and decoding...");

        // Unfortunately we need to use josekit to decrypt the JWE, as biscuit
        // does not yet support A128CBC_HS256 content decoding.
        // See: https://github.com/lawliet89/biscuit/issues/42
        let (jwe_payload, header) = jwe::deserialize_compact(&jwe_token, decrypter)?;

        // Check the "enc" header to confirm that that the content is encoded
        // with the expected algorithm.
        if header.content_encryption() != Some("A128CBC-HS256") {
            // This is the error that would have been returned, if the biscuit
            // crate had done the algorithm checking.
            return Err(biscuit_errors::ValidationError::WrongAlgorithmHeader)?;
        }

        // Get a JWT from the decrypted JWE and decode it using the JWT set.
        let encoded_jwt = JWT::<C, H>::from_bytes(&jwe_payload)?;
        let decoded_jwt = encoded_jwt.decode_with_jwks(jwks, Some(SignatureAlgorithm::RS256))?;

        Ok(decoded_jwt)
    }
}
