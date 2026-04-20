use derive_more::AsRef;
use derive_more::Debug;
use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use itertools::Itertools;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use p256::SecretKey;
use p256::pkcs8::EncodePrivateKey;
use rand_core::OsRng;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::DisplayFromStr;
use serde_with::serde_as;

use crate::algorithm::EcdhAlgorithm;
use crate::algorithm::EncryptionAlgorithm;
use crate::encryption::JwePublicKey;
use crate::error::JweDecryptionError;
use crate::error::JweJsonDecryptionError;
use crate::error::JweStringDecryptionError;

#[cfg(feature = "rsa")]
pub use rsa::JweRsaPrivateKey;

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
        let key = SecretKey::random(&mut OsRng);

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

impl JweDecrypter {
    fn new_ecdh(id: Option<String>, secret_key: &SecretKey, algorithm: EcdhAlgorithm) -> Self {
        let der = secret_key
            .to_pkcs8_der()
            .expect("a p256 secret key should always encode to DER");

        let decrypter = EcdhEsJweAlgorithm::from(algorithm)
            .decrypter_from_der(der.as_bytes())
            .expect("the p256 secret key DER should always be valid");

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
    ) -> Result<Vec<u8>, JweDecryptionError> {
        let (payload, header) =
            josekit::jwe::deserialize_compact(jwe, self.decrypter.as_ref()).map_err(JweDecryptionError::Decryption)?;

        if let Some(id) = self.id.as_deref() {
            let received_id = header.claim("kid").and_then(serde_json::Value::as_str);

            if received_id != Some(id) {
                return Err(JweDecryptionError::IdMismatch(
                    id.to_string(),
                    received_id.map(str::to_string),
                ));
            }
        }

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
    ) -> Result<T, JweJsonDecryptionError>
    where
        T: DeserializeOwned,
    {
        let payload = self.decrypt(jwe, expected_algorithm)?;

        let data = serde_json::from_slice(&payload).map_err(JweJsonDecryptionError::Deserialization)?;

        Ok(data)
    }

    pub fn decrypt_string(
        &self,
        jwe: &str,
        expected_algorithm: ExpectedEncryptionAlgorithm,
    ) -> Result<String, JweStringDecryptionError> {
        let payload = self.decrypt(jwe, expected_algorithm)?;

        let data = String::from_utf8(payload).map_err(JweStringDecryptionError::InvalidUtf8)?;

        Ok(data)
    }
}

#[cfg(feature = "rsa")]
mod rsa {
    use derive_more::Constructor;
    use josekit::jwe::alg::rsaes::RsaesJweAlgorithm;
    use jwk_simple::Key;
    use jwk_simple::KeyParams;
    use jwk_simple::KeyUse;
    use rsa::BigUint;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPrivateKey;

    use crate::algorithm::RsaAlgorithm;
    use crate::error::JwkError;
    use crate::error::RsaPrivateJwkError;

    use super::JweDecrypter;

    #[derive(Debug, Clone, Constructor)]
    pub struct JweRsaPrivateKey {
        id: Option<String>,
        key: RsaPrivateKey,
        algorithm: RsaAlgorithm,
    }

    impl JweRsaPrivateKey {
        pub fn try_from_jwk(jwk: &Key, default_jwe_algorithm: RsaAlgorithm) -> Result<Self, RsaPrivateJwkError> {
            jwk.validate().map_err(JwkError::Invalid)?;

            if let Some(key_use) = jwk.key_use()
                && *key_use != KeyUse::Encryption
            {
                return Err(JwkError::InvalidKeyUse(key_use.clone()).into());
            }

            let (algorithm, jwe_algorithm) = match jwk.alg() {
                None => (&default_jwe_algorithm.into(), default_jwe_algorithm),
                Some(algorithm) => {
                    let jwe_algorithm = RsaAlgorithm::try_from_jwk_simple_algorithm(algorithm)
                        .ok_or(JwkError::UnsupportedAlgorithm(algorithm.clone()))?;

                    (algorithm, jwe_algorithm)
                }
            };

            if !jwk.is_algorithm_compatible(algorithm) {
                return Err(JwkError::InconsistentKeyType(jwk.params().key_type(), algorithm.clone()).into());
            }

            let KeyParams::Rsa(rsa_params) = jwk.params() else {
                unreachable!(
                    "Key::is_algorithm_compatible() in combination with RsaAlgorithm::try_from_jwk_simple_algorithm() \
                     guarantees a supported key type"
                );
            };

            if rsa_params.is_multi_prime() {
                return Err(RsaPrivateJwkError::MultiPrimeUnsupported);
            }

            let modulus = BigUint::from_bytes_be(rsa_params.n.as_bytes());
            let public_exp = BigUint::from_bytes_be(rsa_params.e.as_bytes());
            let private_exp = BigUint::from_bytes_be(
                rsa_params
                    .d
                    .as_ref()
                    .ok_or(RsaPrivateJwkError::MissingPrivateExponent)?
                    .as_bytes(),
            );
            let first_prime = BigUint::from_bytes_be(
                rsa_params
                    .p
                    .as_ref()
                    .ok_or(RsaPrivateJwkError::MissingFirstPrime)?
                    .as_bytes(),
            );
            let second_prime = BigUint::from_bytes_be(
                rsa_params
                    .q
                    .as_ref()
                    .ok_or(RsaPrivateJwkError::MissingSecondPrime)?
                    .as_bytes(),
            );

            let id = jwk.kid().map(str::to_string);

            let private_key =
                RsaPrivateKey::from_components(modulus, public_exp, private_exp, vec![first_prime, second_prime])
                    .map_err(RsaPrivateJwkError::InvalidRsa)?;

            Ok(Self::new(id, private_key, jwe_algorithm))
        }
    }

    impl JweDecrypter {
        fn new_rsa(id: Option<String>, private_key: &RsaPrivateKey, algorithm: RsaAlgorithm) -> Self {
            let der = private_key
                .to_pkcs1_der()
                .expect("a RSA private key should always encode to DER");

            let decrypter = RsaesJweAlgorithm::from(algorithm)
                .decrypter_from_der(der.as_bytes())
                .expect("the RSA private key DER should always be valid");

            Self {
                id,
                decrypter: Box::new(decrypter),
            }
        }

        pub fn from_rsa_private_key(private_key: &JweRsaPrivateKey) -> Self {
            Self::new_rsa(private_key.id.clone(), &private_key.key, private_key.algorithm)
        }
    }

    #[cfg(test)]
    mod tests {
        use jwk_simple::Key;
        use serde_json::json;

        use crate::algorithm::RsaAlgorithm;

        use super::JweDecrypter;
        use super::JweRsaPrivateKey;

        #[test]
        fn test_jwe_rsa_private_key_try_from_jwk() {
            let jwk_json = json!({
                "kty": "RSA",
                "kid": "cc34c0a0-bd5a-4a3c-a50d-a2a7db7643df",
                "alg": "RSA-OAEP-512",
                "n": "pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGa\
                      ut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByy\
                      FKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS\
                      2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk\
                      9_B_f1lVrmOKuHjTNHq48w",
                "e": "AQAB",
                "d": "ksDmucdMJXkFGZxiomNHnroOZxe8AmDLDGO1vhs-POa5PZM7mtUPonxwjVmthmpbZzla-kg55OFfO7Yc\
                      Xhg-Hm2OWTKwm73_rLh3JavaHjvBqsVKuorX3V3RYkSro6HyYIzFJ1Ek7sLxbjDRcDOj4ievSX0oN9l-\
                      JZhaDYlPlci5uJsoqro_YrE0PRRWVhtGynd-_aWgQv1YzkfZuMD-hJtDi1Im2humOWxA4eZrFs9eG-wh\
                      XcOvaSwO4sSGbS99ecQZHM2TcdXeAs1PvjVgQ_dKnZlGN3lTWoWfQP55Z7Tgt8Nf1q4ZAKd-NlMe-7iq\
                      CFfsnFwXjSiaOa2CRGZn-Q",
                "p": "4A5nU4ahEww7B65yuzmGeCUUi8ikWzv1C81pSyUKvKzu8CX41hp9J6oRaLGesKImYiuVQK47FhZ--wwf\
                      pRwHvSxtNU9qXb8ewo-BvadyO1eVrIk4tNV543QlSe7pQAoJGkxCia5rfznAE3InKF4JvIlchyqs0RQ8\
                      wx7lULqwnn0",
                "q": "ven83GM6SfrmO-TBHbjTk6JhP_3CMsIvmSdo4KrbQNvp4vHO3w1_0zJ3URkmkYGhz2tgPlfd7v1l2I6Q\
                      kIh4Bumdj6FyFZEBpxjE4MpfdNVcNINvVj87cLyTRmIcaGxmfylY7QErP8GFA-k4UoH_eQmGKGK44TRz\
                      Yj5hZYGWIC8",
                "dp": "lmmU_AG5SGxBhJqb8wxfNXDPJjf__i92BgJT2Vp4pskBbr5PGoyV0HbfUQVMnw977RONEurkR6O6gxZU\
                       eCclGt4kQlGZ-m0_XSWx13v9t9DIbheAtgVJ2mQyVDvK4m7aRYlEceFh0PsX8vYDS5o1txgPwb3oXkPT\
                       trmbAGMUBpE",
                "dq": "mxRTU3QDyR2EnCv0Nl0TCF90oliJGAHR9HJmBe__EjuCBbwHfcT8OG3hWOv8vpzokQPRl5cQt3NckzX3\
                       fs6xlJN4Ai2Hh2zduKFVQ2p-AF2p6Yfahscjtq-GY9cB85NxLy2IXCC0PF--Sq9LOrTE9QV988SJy_yU\
                       rAjcZ5MmECk",
                "qi": "ldHXIrEmMZVaNwGzDF9WG8sHj2mOZmQpw9yrjLK9hAsmsNr5LTyqWAqJIYZSwPTYWhY4nu2O0EY9G9uY\
                       iqewXfCKw_UngrJt8Xwfq1Zruz0YY869zPN4GiE9-9rzdZB33RBw8kIOquY3MK74FMwCihYx_LiU2YTH\
                       kaoJ3ncvtvg"
            });
            let jwk = serde_json::from_value::<Key>(jwk_json).unwrap();

            let private_key = JweRsaPrivateKey::try_from_jwk(&jwk, RsaAlgorithm::RsaOaep)
                .expect("converting from JWK to JweRsaPrivateKey should succeed");

            let _decrypter = JweDecrypter::from_rsa_private_key(&private_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use jwk_simple::KeyParams;
    use jwk_simple::jwk::Key;
    use p256::SecretKey;
    use serde_json::json;

    use crate::algorithm::EcdhAlgorithm;

    use super::ExpectedEncryptionAlgorithm;
    use super::JweDecrypter;
    use super::JweEcdhSecretKey;

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

    #[test]
    fn test_jwe_decrypter() {
        let jwk = serde_json::from_value::<Key>(example_jwk()).unwrap();

        let id = jwk.kid().map(str::to_string);
        let secret_key = jwk_to_secret_key(&jwk);
        let algorithm = EcdhAlgorithm::try_from_jwk_simple_algorithm(jwk.alg().unwrap()).unwrap();

        let key = JweEcdhSecretKey::new(id, secret_key, algorithm);
        let decrypter = JweDecrypter::from_ecdh_secret_key(&key);

        let data = decrypter
            .decrypt_json::<serde_json::Value>(EXAMPLE_JWE, ExpectedEncryptionAlgorithm::Any)
            .expect("decrypting example JWE as JSON should succeed");

        assert_eq!(data, example_jwe_contents());

        let _ = decrypter
            .decrypt_string(EXAMPLE_JWE, ExpectedEncryptionAlgorithm::Any)
            .expect("decrypting example JWE as string should succeed");
    }
}
