use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::Path,
    time::Duration,
};

use futures::future::TryFutureExt;
use http::header;
use josekit::{
    jwe::{self, alg::rsaes::RsaesJweDecrypter},
    jwk::Jwk,
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

const APPLICATION_JWT: &str = "application/jwt";
const BSN_KEY: &str = "uzi_id";

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
    JoseKit(#[from] JoseError),
}

impl From<biscuit_errors::Error> for Error {
    fn from(value: biscuit_errors::Error) -> Self {
        openid_errors::Error::from(value).into()
    }
}

impl From<biscuit_errors::ValidationError> for Error {
    fn from(value: biscuit_errors::ValidationError) -> Self {
        biscuit_errors::Error::from(value).into()
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        openid_errors::Error::from(value).into()
    }
}

impl From<openid_errors::ClientError> for Error {
    fn from(value: openid_errors::ClientError) -> Self {
        openid_errors::Error::from(value).into()
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        openid_errors::ClientError::from(value).into()
    }
}

impl From<openid_errors::Userinfo> for Error {
    fn from(value: openid_errors::Userinfo) -> Self {
        openid_errors::Error::from(value).into()
    }
}

pub struct Client(openid::Client);

impl Client {
    pub async fn discover(issuer_url: Url, client_id: impl Into<String>) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(CLIENT_TIMEOUT)
            .build()
            .expect("Could not build reqwest HTTP client");
        let client =
            openid::Client::discover_with_client(http_client, client_id.into(), None, None, issuer_url).await?;

        Ok(Client(client))
    }

    pub fn decrypter_from_jwk_file(path: impl AsRef<Path>) -> Result<RsaesJweDecrypter> {
        // Open the file in read-only mode with buffer.
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as JWK,
        // the create a decrypter from it.
        let jwk: Jwk = serde_json::from_reader(reader)?;
        let decrypter = jwe::RSA_OAEP.decrypter_from_jwk(&jwk)?;

        Ok(decrypter)
    }

    pub fn bsn_from_claims(userinfo_token: &UserInfoJWT) -> Result<Option<String>> {
        debug!("userinfo_token: {:?}", userinfo_token);

        let userinfo_payload = userinfo_token.payload()?;

        debug!("Registered Claims: {:?}", userinfo_payload.registered);

        userinfo_payload.registered.validate(ValidationOptions::default())?;

        for (key, value) in userinfo_payload.private.iter() {
            debug!("Private Claim {}: {}", key, value);
        }

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

        // The JWK set should always be populated by discovery.
        let jwks = self
            .0
            .jwks
            .as_ref()
            .expect("OpenID client JWK set not populated by disovery");

        // Get userinfo endpoint from discovery, throw an error otherwise.
        let endpoint = self
            .0
            .config()
            .userinfo_endpoint
            .as_ref()
            .cloned()
            .ok_or(openid_errors::Userinfo::NoUrl)?;

        // Use the access_token to retrieve the userinfo as a JWE token.
        let jwe_token = self
            .0
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
