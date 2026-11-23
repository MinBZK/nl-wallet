use std::sync::LazyLock;

use derive_more::AsRef;
use derive_more::Debug;
use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use itertools::Itertools;
use josekit::jwe::JweContext;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use p256::SecretKey;
use p256::elliptic_curve::Generate;
use p256::pkcs8::EncodePrivateKey;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::DisplayFromStr;
use serde_with::serde_as;

use crate::algorithm::CompressionAlgorithm;
use crate::algorithm::EcdhAlgorithm;
use crate::algorithm::EncryptionAlgorithm;
use crate::encryption::JwePublicKey;
use crate::error::JweDecryptionError;
use crate::error::JweJsonDecryptionError;
use crate::error::JweStringDecryptionError;

static NO_DEFLATE_JWE_CONTEXT: LazyLock<JweContext> = LazyLock::new(|| {
    let mut context = JweContext::new();
    context.remove_compression(&CompressionAlgorithm::Deflate.to_string());

    context
});

#[derive(Debug, Clone, From, AsRef, Display, FromStr)]
#[display(
    "{}",
    self
        .0
        .to_pkcs8_pem(Default::default())
        .expect("a p256 secret key should always encode to PKCS #8 PEM")
        .as_str()
)]
struct PemSecretKey(SecretKey);

/// Wraps a P-256 EC secret key, an optional `kid` value and a JWE algorithm. This type is meant to be converted to a
/// [`JwePublicKey`], which can then be converted to a JWK in the form of a [`jwk_simple::jwk::Key`] and sent to
/// another party. JWEs from this other party can then be decrypted by converting it into a [`JweDecrypter`].
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JweEcdhSecretKey {
    id: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    key: PemSecretKey,
    #[serde_as(as = "DisplayFromStr")]
    algorithm: EcdhAlgorithm,
}

impl JweEcdhSecretKey {
    pub fn new(id: Option<String>, key: SecretKey, algorithm: EcdhAlgorithm) -> Self {
        Self {
            id,
            key: key.into(),
            algorithm,
        }
    }

    pub fn new_random(id: Option<String>, algorithm: EcdhAlgorithm) -> Self {
        let key = SecretKey::generate();

        Self::new(id, key, algorithm)
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn key(&self) -> &SecretKey {
        self.key.as_ref()
    }

    pub fn algorithm(&self) -> EcdhAlgorithm {
        self.algorithm
    }

    pub fn to_jwe_public_key(&self) -> JwePublicKey {
        JwePublicKey::new(self.id.clone(), self.key.as_ref().public_key(), self.algorithm)
    }
}

/// Wraps JWE decryption using the key that is derived from an eliptic curve P-256 public key and optional `kid` value.
/// Can be constructed from a [`JweSecretKey`].
#[derive(Debug, Clone)]
pub struct JweDecrypter {
    id: Option<String>,
    decrypter: Box<dyn josekit::jwe::JweDecrypter>,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedEncryptionAlgorithm<'a> {
    Any,
    Algorithms(&'a [EncryptionAlgorithm]),
}

#[derive(Debug, Clone, Copy)]
pub enum AllowedDecompression {
    None,
    Deflate,
}

impl JweDecrypter {
    fn new_ecdh(id: Option<String>, secret_key: &SecretKey, algorithm: EcdhAlgorithm) -> Self {
        let der = secret_key
            .to_pkcs8_der()
            .expect("a p256 secret key should always encode to DER");

        let mut decrypter = EcdhEsJweAlgorithm::from(algorithm)
            .decrypter_from_der(der.as_bytes())
            .expect("the p256 secret key DER should always be valid");

        if let Some(id) = &id {
            decrypter.set_key_id(id.clone());
        }

        Self {
            id,
            decrypter: Box::new(decrypter),
        }
    }

    pub fn from_ecdh_secret_key(secret_key: &JweEcdhSecretKey) -> Self {
        Self::new_ecdh(secret_key.id.clone(), secret_key.key.as_ref(), secret_key.algorithm)
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn decrypt(
        &self,
        jwe: &str,
        expected_algorithm: ExpectedEncryptionAlgorithm,
        allowed_decompression: AllowedDecompression,
    ) -> Result<Vec<u8>, JweDecryptionError> {
        let (payload, header) = match allowed_decompression {
            AllowedDecompression::Deflate => josekit::jwe::deserialize_compact(jwe, self.decrypter.as_ref()),
            AllowedDecompression::None => NO_DEFLATE_JWE_CONTEXT.deserialize_compact(jwe, self.decrypter.as_ref()),
        }
        .map_err(JweDecryptionError::Decryption)?;

        if let ExpectedEncryptionAlgorithm::Algorithms(algorithms) = expected_algorithm {
            let enc = header
                .content_encryption()
                .expect("decryption should have failed without \"enc\" header claim");

            if !algorithms.iter().map(ToString::to_string).contains(enc) {
                return Err(JweDecryptionError::UnexpectedEncryptionAlgorithm {
                    received: enc.to_string(),
                    expected: algorithms.to_vec(),
                });
            }
        }

        Ok(payload)
    }

    pub fn decrypt_json<T>(
        &self,
        jwe: &str,
        expected_algorithm: ExpectedEncryptionAlgorithm,
        allowed_decompression: AllowedDecompression,
    ) -> Result<T, JweJsonDecryptionError>
    where
        T: DeserializeOwned,
    {
        let payload = self.decrypt(jwe, expected_algorithm, allowed_decompression)?;

        let data = serde_json::from_slice(&payload).map_err(JweJsonDecryptionError::Deserialization)?;

        Ok(data)
    }

    pub fn decrypt_string(
        &self,
        jwe: &str,
        expected_algorithm: ExpectedEncryptionAlgorithm,
        allowed_decompression: AllowedDecompression,
    ) -> Result<String, JweStringDecryptionError> {
        let payload = self.decrypt(jwe, expected_algorithm, allowed_decompression)?;

        let data = String::from_utf8(payload).map_err(JweStringDecryptionError::InvalidUtf8)?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use jwk_simple::KeyParams;
    use jwk_simple::jwk::Key;
    use p256::SecretKey;
    use rstest::rstest;
    use serde_json::json;

    use super::AllowedDecompression;
    use super::ExpectedEncryptionAlgorithm;
    use super::JweDecrypter;
    use super::JweEcdhSecretKey;
    use crate::algorithm::EcdhAlgorithm;

    // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.3-7
    const EXAMPLE_JWE: &str = "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTEyOEdDTSIsImtpZCI6ImFjIiwiZXBrIjp7Imt0eSI6IkVD\
                               IiwieCI6Im5ubVZwbTNWM2piaGNhZlFhUkJrU1ZOSGx3Wkh3dC05ck9wSnVmeVlJdWsiLCJ5IjoicjRm\
                               akRxd0p5czlxVU9QLV9iM21SNVNaRy0tQ3dPMm1pYzVWU05UWU45ZyIsImNydiI6IlAtMjU2In19..uA\
                               YcHRUSSn2X0WPX.yVzlGSYG4qbg0bq18JcUiDRw56yVnbKR8E7S7YlEtzT00RqE3Pw5oTpUG3hdLN4ta\
                               HZ9gC1kwak8JOnJgQ.1wR024_3-qtAlx1oFIUpQQ";

    fn example_jwk() -> serde_json::Value {
        // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.3-7
        json!({
            "alg": "ECDH-ES",
            "crv": "P-256",
            "d": "Et-3ce0omz8_TuZ96Df9lp0GAaaDoUnDe6X-CRO7Aww",
            "kid": "ac",
            "kty": "EC",
            "use": "enc",
            "x": "YO4epjifD-KWeq1sL2tNmm36BhXnkJ0He-WqMYrp9Fk",
            "y": "Hekpm0zfK7C-YccH5iBjcIXgf6YdUvNUac_0At55Okk"
        })
    }

    fn example_jwe_contents() -> serde_json::Value {
        // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.3-13
        json!({
            "vp_token": {
                "example_credential_id": [
                    "eyJhb...YMetA"
                ]
            }
        })
    }

    fn jwk_to_secret_key(jwk: &Key) -> SecretKey {
        let KeyParams::Ec(params) = jwk.params() else {
            panic!();
        };

        SecretKey::from_slice(params.d.as_ref().unwrap().as_bytes()).unwrap()
    }

    #[rstest]
    fn test_jwe_decrypter(
        #[values(AllowedDecompression::None, AllowedDecompression::Deflate)]
        allowed_decompression: AllowedDecompression,
    ) {
        let jwk = serde_json::from_value::<Key>(example_jwk()).unwrap();

        let id = jwk.kid().map(str::to_string);
        let secret_key = jwk_to_secret_key(&jwk);
        let algorithm = EcdhAlgorithm::try_from_jwk_simple_algorithm(jwk.alg().unwrap()).unwrap();

        let key = JweEcdhSecretKey::new(id, secret_key, algorithm);
        let decrypter = JweDecrypter::from_ecdh_secret_key(&key);

        let data = decrypter
            .decrypt_json::<serde_json::Value>(EXAMPLE_JWE, ExpectedEncryptionAlgorithm::Any, allowed_decompression)
            .expect("decrypting example JWE as JSON should succeed");

        assert_eq!(data, example_jwe_contents());

        let _ = decrypter
            .decrypt_string(EXAMPLE_JWE, ExpectedEncryptionAlgorithm::Any, allowed_decompression)
            .expect("decrypting example JWE as string should succeed");
    }
}
