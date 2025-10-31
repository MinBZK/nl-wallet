use std::sync::LazyLock;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use derive_more::Display;
use derive_more::FromStr;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Validation;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crypto::CredentialEcdsaKey;
use crypto::EcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use jwt::EcdsaDecodingKey;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::VerifiedJwt;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_to_p256;
use utils::generator::Generator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::error::KeyBindingError;
use crate::error::SigningError;
use crate::hasher::Hasher;
use crate::sd_jwt::SdJwtClaims;
use crate::sd_jwt::VerifiedSdJwt;

// <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-22.html#section-4.3-3.1.2.1>
pub const KB_JWT_HEADER_TYP: &str = "kb+jwt";

impl JwtTyp for KeyBindingJwtClaims {
    const TYP: &'static str = KB_JWT_HEADER_TYP;
}

/// Verification options for KB-JWT verification:
/// - `expected_aud`: audience to enforce,
/// - `expected_nonce`: nonce to match,
/// - `iat_leeway`: allowed leeway around the lower bound of `iat`,
/// - `iat_acceptance_window`: allowed duration after `iat`.
pub struct KbVerificationOptions<'a> {
    pub expected_aud: &'a str,
    pub expected_nonce: &'a str,
    pub iat_leeway: Duration,
    pub iat_acceptance_window: Duration,
}

/// Representation of a [KB-JWT](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-12.html#name-key-binding-jwt).
///
/// Implemented as a wrapper around `UnverifiedJwt`. Can be verified using `into_verified`.
#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr, Serialize, Deserialize)]
pub struct UnverifiedKeyBindingJwt(UnverifiedJwt<KeyBindingJwtClaims>);

/// Verified KB-JWT (claims parsed and signature validated).
pub type VerifiedKeyBindingJwt = VerifiedJwt<KeyBindingJwtClaims>;

/// Freshly signed KB-JWT.
pub type SignedKeyBindingJwt = SignedJwt<KeyBindingJwtClaims>;

impl UnverifiedKeyBindingJwt {
    /// Verifies the KB-JWT by checking the signature using the provided public key and validation options.
    ///
    /// Additionally;
    /// - enforces expected `aud`
    /// - verifies expected `nonce`
    /// - checks that `iat` is within the acceptance window (with leeway),
    ///
    /// <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-12.html#section-8.3-4.5.1>
    pub fn into_verified(
        self,
        pubkey: &EcdsaDecodingKey,
        kb_verification_options: &KbVerificationOptions,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VerifiedKeyBindingJwt, KeyBindingError> {
        let validation_options = kb_jwt_validation(kb_verification_options.expected_aud);
        let verified = self.0.into_verified(pubkey, &validation_options)?;

        let payload = verified.payload();
        if payload.nonce != kb_verification_options.expected_nonce {
            return Err(KeyBindingError::NonceMismatch(payload.nonce.clone()));
        };

        let now = time.generate();
        let leeway = kb_verification_options.iat_leeway;
        if !(payload.iat <= now + leeway && now <= payload.iat + kb_verification_options.iat_acceptance_window) {
            return Err(KeyBindingError::InvalidSignatureTimestamp(
                payload.iat,
                kb_verification_options.iat_acceptance_window,
                now,
            ));
        };

        Ok(verified)
    }

    pub fn from_signed(signed: SignedKeyBindingJwt) -> Self {
        Self(signed.into())
    }
}

fn kb_jwt_validation(expected_aud: &str) -> Validation {
    let mut validation = BASE_KB_JWT_VALIDATION.to_owned();
    validation.set_audience(&[expected_aud]);
    validation
}

static BASE_KB_JWT_VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_aud = true;
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.set_required_spec_claims(&["aud"]);
    validation
});

/// Builder-style struct to ease the creation of a [`SignedKeyBindingJwt`].
#[derive(Debug, Clone)]
pub struct KeyBindingJwtBuilder {
    aud: String,
    nonce: String,
}

impl KeyBindingJwtBuilder {
    pub fn new(aud: String, nonce: String) -> Self {
        Self { aud, nonce }
    }

    fn sd_hash_for_sd_jwt<C: SdJwtClaims, H>(sd_jwt: &VerifiedSdJwt<C, H>) -> Result<String, KeyBindingError> {
        let hasher = sd_jwt.claims()._sd_alg().unwrap_or_default().hasher()?;

        let sd_hash = hasher.encoded_digest(&sd_jwt.to_string());

        Ok(sd_hash)
    }

    /// Builds an [`KeyBindingJwt`] from the data provided to builder.
    pub(crate) async fn finish<C: SdJwtClaims, H>(
        self,
        sd_jwt: &VerifiedSdJwt<C, H>,
        signing_key: &impl EcdsaKey,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<SignedKeyBindingJwt, KeyBindingError> {
        let sd_hash = Self::sd_hash_for_sd_jwt(sd_jwt)?;

        let claims = KeyBindingJwtClaims {
            iat: time.generate(),
            aud: self.aud,
            nonce: self.nonce,
            sd_hash,
        };

        let signed_jwt = SignedJwt::sign(&claims, signing_key).await?;
        Ok(signed_jwt)
    }

    /// Builds several [`KeyBindingJwt`]s from the data provided by one builder, using the WSCD.
    pub(crate) async fn finish_multiple<C, H, K, W, P>(
        self,
        sd_jwts_and_keys: &VecNonEmpty<(VerifiedSdJwt<C, H>, K)>,
        wscd: &W,
        poa_input: P::Input,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<(VecNonEmpty<SignedKeyBindingJwt>, Option<P>), SigningError>
    where
        C: SdJwtClaims,
        W: DisclosureWscd<Key = K, Poa = P>,
        K: CredentialEcdsaKey,
        P: WscdPoa,
    {
        // Create a `KeyBindingJwtClaims` for each `SdJwt`, based on the contents of the builder and combine it with the
        // provided key.
        let iat = time.generate();

        let sd_jwt_count = sd_jwts_and_keys.len().get();
        let payloads_and_keys: VecNonEmpty<_> = sd_jwts_and_keys
            .into_iter()
            .zip(itertools::repeat_n(self, sd_jwt_count))
            .map(|((sd_jwt, key), builder)| {
                let KeyBindingJwtBuilder { aud, nonce } = builder;
                let sd_hash = Self::sd_hash_for_sd_jwt(sd_jwt)?;

                let claims = KeyBindingJwtClaims {
                    iat,
                    aud,
                    nonce,
                    sd_hash,
                };

                Ok((claims, key))
            })
            .collect::<Result<Vec<_>, KeyBindingError>>()?
            .try_into()
            .unwrap();

        // Create JWTs from all of these by having the WSCD sign the `KeyBindingJwtClaims` values.
        let (signed_jwts, poa) = SignedJwt::sign_multiple(
            payloads_and_keys.nonempty_iter().map(|(payload, key)| (payload, *key)),
            wscd,
            poa_input,
        )
        .await?;

        Ok((signed_jwts, poa))
    }
}

/// Claims set for key binding JWT.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct KeyBindingJwtClaims {
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
    pub aud: String,
    pub nonce: String,
    pub sd_hash: String,
}

/// Proof of possession of a given key.
///
/// Currently, only Jwk is supported. See [RFC7800](https://www.rfc-editor.org/rfc/rfc7800.html#section-3) for more
/// details.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequiredKeyBinding {
    /// Json Web Key (JWK).
    Jwk(Jwk),
}

impl RequiredKeyBinding {
    pub fn verifying_key(&self) -> Result<VerifyingKey, JwkConversionError> {
        let Self::Jwk(jwk) = self;
        jwk_to_p256(jwk)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use base64::Engine;
    use base64::prelude::*;
    use chrono::TimeZone;
    use chrono::Utc;
    use futures::FutureExt;
    use itertools::Itertools;
    use jsonwebtoken::Algorithm;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde_json::json;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use crypto::server_keys::generate::Ca;
    use jwt::EcdsaDecodingKey;
    use jwt::SignedJwt;
    use jwt::error::JwtError;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::IntoNonEmptyIterator;
    use utils::vec_at_least::NonEmptyIterator;
    use utils::vec_nonempty;

    use crate::builder::SdJwtBuilder;
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::sd_jwt::SdJwtVcClaims;

    use super::*;

    async fn example_kb_jwt(signing_key: &SigningKey) -> SignedJwt<KeyBindingJwtClaims> {
        example_kb_jwt_with_iat(
            signing_key,
            Utc::now() - Duration::from_secs(2 * 24 * 60 * 60), // 2 days ago
        )
        .await
    }

    async fn example_kb_jwt_with_iat(signing_key: &SigningKey, iat: DateTime<Utc>) -> SignedJwt<KeyBindingJwtClaims> {
        SignedJwt::sign(
            &KeyBindingJwtClaims {
                iat,
                aud: String::from("aud"),
                nonce: String::from("abc123"),
                sd_hash: String::from("sd_hash"),
            },
            signing_key,
        )
        .await
        .unwrap()
    }

    fn header_and_payload_values_for_kb_jwt(
        signed_jwt: &SignedKeyBindingJwt,
    ) -> (serde_json::Value, serde_json::Value) {
        signed_jwt
            .as_ref()
            .signed_slice()
            .split('.')
            .map(|base64| {
                let json = String::try_from(BASE64_URL_SAFE_NO_PAD.decode(base64).unwrap()).unwrap();

                serde_json::from_str::<serde_json::Value>(&json).unwrap()
            })
            .collect_tuple()
            .unwrap()
    }

    #[test]
    fn test_key_binding_jwt_builder() {
        let sd_jwt = VerifiedSdJwt::spec_sd_jwt_vc();

        let signing_key = SigningKey::random(&mut OsRng);
        let hasher = Sha256Hasher;

        let time = MockTimeGenerator::new(Utc::now());

        let kb_jwt = KeyBindingJwtBuilder::new(String::from("receiver"), String::from("abc123"))
            .finish(&sd_jwt, &signing_key, &time)
            .now_or_never()
            .unwrap()
            .expect("signing a KeyBindingJwt should succeed");

        let (header, payload) = header_and_payload_values_for_kb_jwt(&kb_jwt);

        let expected_header = json!({
            "typ": "kb+jwt",
            "alg": Algorithm::ES256
        });

        let sd_hash = hasher.encoded_digest(sd_jwt.to_string().as_str());
        let expected_payload = json!({
            "iat": time.generate().timestamp(),
            "aud": "receiver",
            "nonce": "abc123",
            "sd_hash": sd_hash,
        });

        assert_eq!(header, expected_header);
        assert_eq!(payload, expected_payload);
    }

    #[test]
    fn test_key_binding_jwt_builder_multi() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();
        let key1 = MockRemoteEcdsaKey::new_random("key1".to_string());
        let key2 = MockRemoteEcdsaKey::new_random("key2".to_string());
        let wscd = MockRemoteWscd::new(vec![key1.clone(), key2.clone()]);

        let iat = Utc::now();
        let mock_time = MockTimeGenerator::new(iat);

        let sd_jwts_and_keys = vec_nonempty![("Doe", key1), ("Deer", key2)]
            .into_nonempty_iter()
            .map(|(family_name, key)| {
                // Create a minimal SD-JWT that contains the holder verifying key as JWK.
                let sd_jwt = SdJwtBuilder::new(SdJwtVcClaims::example_from_json(
                    key.verifying_key(),
                    json!({ "family_name": family_name}),
                    &mock_time,
                ))
                .finish(&issuer_keypair)
                .now_or_never()
                .unwrap()
                .unwrap()
                .into_verified();

                (sd_jwt, key)
            })
            .collect();

        let (kb_jwts, poa) = KeyBindingJwtBuilder::new(String::from("receiver"), String::from("abc123"))
            .finish_multiple(&sd_jwts_and_keys, &wscd, (), &mock_time)
            .now_or_never()
            .unwrap()
            .expect("signing multiple KeyBindingJwt values using WSCD should succeed");

        assert!(poa.is_some());

        let hasher = Sha256Hasher;

        for (sd_jwt, kb_jwt) in sd_jwts_and_keys
            .iter()
            .zip_eq(kb_jwts.iter())
            .map(|((sd_jwt, _), kb_jwt)| (sd_jwt, kb_jwt))
        {
            let (header, payload) = header_and_payload_values_for_kb_jwt(kb_jwt);

            let expected_header = json!({
                "typ": "kb+jwt",
                "alg": Algorithm::ES256
            });

            let sd_hash = hasher.encoded_digest(sd_jwt.to_string().as_str());
            let expected_payload = json!({
                "iat": iat.timestamp(),
                "aud": "receiver",
                "nonce": "abc123",
                "sd_hash": sd_hash,
            });

            assert_eq!(header, expected_header);
            assert_eq!(payload, expected_payload);
        }
    }

    #[tokio::test]
    async fn test_parse_should_validate() {
        let signing_key = SigningKey::random(&mut OsRng);

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "aud",
            expected_nonce: "abc123",
            iat_leeway: Duration::ZERO,
            iat_acceptance_window: Duration::from_secs(3 * 24 * 60 * 60),
        };

        let jwt_str = example_kb_jwt(&signing_key).await.to_string();
        jwt_str
            .parse::<UnverifiedKeyBindingJwt>()
            .unwrap()
            .into_verified(
                &EcdsaDecodingKey::from(signing_key.verifying_key()),
                &kb_verification_options,
                &MockTimeGenerator::default(),
            )
            .unwrap();
    }

    #[rstest]
    #[case::not_yet_valid(1000, Duration::from_secs(5), 994, Duration::from_secs(500), false)]
    #[case::valid_in_leeway(1000, Duration::from_secs(5), 995, Duration::from_secs(500), true)]
    #[case::valid(1000, Duration::from_secs(5), 1200, Duration::from_secs(500), true)]
    #[case::valid_atwindow_boundary(1000, Duration::from_secs(5), 1500, Duration::from_secs(500), true)]
    #[case::expired(1000, Duration::from_secs(5), 1501, Duration::from_secs(500), false)]
    #[tokio::test]
    async fn test_parse_and_verify_iat(
        #[case] iat_epoch: i64,
        #[case] leeway: Duration,
        #[case] now_epoch: i64,
        #[case] iat_acceptance_window: Duration,
        #[case] expected_valid: bool,
    ) {
        let signing_key = SigningKey::random(&mut OsRng);

        let iat_generator = MockTimeGenerator::new(Utc.timestamp_opt(iat_epoch, 0).unwrap());
        let iat = iat_generator.generate();

        let now_generator = MockTimeGenerator::new(Utc.timestamp_opt(now_epoch, 0).unwrap());

        let jwt_str = example_kb_jwt_with_iat(&signing_key, iat).await.to_string();

        let verify_timestamp = |iat: DateTime<Utc>, window: Duration, current_time: DateTime<Utc>| {
            window == iat_acceptance_window
                && iat == iat_generator.generate()
                && current_time == now_generator.generate()
        };

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "aud",
            expected_nonce: "abc123",
            iat_leeway: leeway,
            iat_acceptance_window,
        };

        let result = jwt_str.parse::<UnverifiedKeyBindingJwt>().unwrap().into_verified(
            &EcdsaDecodingKey::from(signing_key.verifying_key()),
            &kb_verification_options,
            &now_generator,
        );

        if expected_valid {
            let _verified_jwt = result.unwrap();
        } else {
            let err = result.unwrap_err();
            assert_matches!(err, KeyBindingError::InvalidSignatureTimestamp(iat, window, now)
                        if verify_timestamp(iat, window, now));
        }
    }

    #[tokio::test]
    async fn test_parse_should_error_for_wrong_nonce() {
        let signing_key = SigningKey::random(&mut OsRng);

        let jwt_str = example_kb_jwt(&signing_key).await.to_string();

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "aud",
            expected_nonce: "def456",
            iat_leeway: Duration::ZERO,
            iat_acceptance_window: Duration::from_secs(3 * 24 * 60 * 60),
        };

        let err = jwt_str
            .parse::<UnverifiedKeyBindingJwt>()
            .unwrap()
            .into_verified(
                &EcdsaDecodingKey::from(signing_key.verifying_key()),
                &kb_verification_options,
                &MockTimeGenerator::default(),
            )
            .expect_err("should fail validation");
        assert_matches!(
            err,
            KeyBindingError::NonceMismatch(actual) if &actual == "abc123"
        );
    }

    #[tokio::test]
    async fn test_parse_should_error_for_invalid_audience() {
        let signing_key = SigningKey::random(&mut OsRng);

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "other_aud",
            expected_nonce: "abc123",
            iat_leeway: Duration::ZERO,
            iat_acceptance_window: Duration::from_secs(3 * 24 * 60 * 60),
        };

        let jwt_str = example_kb_jwt(&signing_key).await.to_string();
        let err = jwt_str
            .parse::<UnverifiedKeyBindingJwt>()
            .unwrap()
            .into_verified(
                &EcdsaDecodingKey::from(signing_key.verifying_key()),
                &kb_verification_options,
                &MockTimeGenerator::default(),
            )
            .expect_err("should fail validation");
        assert_matches!(
            err,
            KeyBindingError::Jwt(JwtError::Validation(error))
                if *error.kind() == jsonwebtoken::errors::ErrorKind::InvalidAudience
        );
    }
}
