use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::EcdsaKey;
use crypto::PublicKey;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateUsage;
use derive_more::AsRef;
use derive_more::Constructor;
use error_category::ErrorCategory;
use itertools::Itertools;
use jwt::Algorithm;
use jwt::JwtTyp;
use jwt::JwtValidation;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwkConversionError;
use jwt::error::JwtSignError;
use jwt::error::JwtX5cVerifyError;
use jwt::headers::HeaderWithTyp;
use jwt::headers::HeaderWithX5c;
use jwt::jwk::Jwk;
use jwt::jwk::jwk_from_public_key;
use jwt::jwk::jwk_to_public_key;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::skip_serializing_none;
use strum::EnumString;
use url::Url;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;

pub const OPENID4VCI_PROOF_JWT_TYP: &str = "key-attestation+jwt";

/// Used to set the `exp` value of `KeyAttestationClaims`.
const VALIDITY_TIME: Duration = Duration::from_mins(15);

static ES256_JWT_VALIDATION: LazyLock<JwtValidation> =
    LazyLock::new(|| JwtValidation::default_with_algorithms([Algorithm::ES256]));

/// The JOSE header for the OpenID4VCI Key Attestation.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1>
///
/// This contains the following fields:
/// - The `alg` and `typ` header are provided by `HeaderWithTyp`.
/// - The `x5c` header is provided by `HeaderWithX5c`.
///
/// Note that HAIP states that the key attestation MUST include the `x5c` header. For this reason we can ignore the
/// `kid` and `trust_chain` fields mentioned in OpenID4VCI as alternatives.
pub type KeyAttestationHeader = HeaderWithX5c<HeaderWithTyp>;

#[derive(Debug, thiserror::Error)]
pub enum KeyAttestationSigningError {
    #[error("could not convert public key to JWK: {0}")]
    PublicKey(#[source] JwkConversionError),

    #[error("could not sign JWT: {0}")]
    SignJwt(#[source] JwtSignError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum KeyAttestationVerificationError {
    #[error("invalid key attestation JWT: {0}")]
    InvalidJwt(#[source] JwtX5cVerifyError),

    #[error("could not extract public key from JWK: {0}")]
    InvalidJwk(#[source] JwkConversionError),

    #[error("attested keys contain duplicate values")]
    #[category(critical)]
    AttestedKeysNotDistinct,

    #[error("key storage attack potential resistance not sufficient, found: {}", .0.iter().join(", "))]
    #[category(critical)]
    KeyStorageAttackResistance(HashSet<AttackPotentialResistance>),

    #[error("user authentication attack potential resistance not sufficient, found: {}", .0.iter().join(", "))]
    #[category(critical)]
    UserAuthenticationAttackResistance(HashSet<AttackPotentialResistance>),
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRef)]
pub struct KeyAttestation(UnverifiedJwt<KeyAttestationClaims, KeyAttestationHeader>);

/// The JWT claims for an OpenID4VCI Key Attestation, to be used by the wallet as proof of cryptographic key material in
/// a Credential Request.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttestationClaims {
    /// REQUIRED (number). Integer for the time at which the key attestation was issued using the syntax defined in
    /// RFC7519.
    pub iat: DateTimeSeconds,

    /// OPTIONAL (number). Integer for the time at which the key attestation and the key(s) it is attesting expire,
    /// using the syntax defined in RFC7519. MUST be present if the attestation is used with the JWT proof type.
    pub exp: Option<DateTimeSeconds>,

    /// REQUIRED. A non-empty array of attested keys from the same key storage component using the syntax of JWK as
    /// defined in RFC7517.
    pub attested_keys: VecNonEmpty<Jwk>,

    /// OPTIONAL. A non-empty array of case sensitive strings that assert the attack potential resistance of the key
    /// storage component and its keys attested in the attested_keys parameter.
    ///
    /// Note that this uses a `HashSet`, which makes more sense semantically than a `Vec`.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub key_storage: HashSet<AttackPotentialResistance>,

    /// OPTIONAL. A non-empty array of case sensitive strings that assert the attack potential resistance of the user
    /// authentication methods allowed to access the private keys from the attested_keys parameter.
    ///
    /// Note that this uses a `HashSet`, which makes more sense semantically than a `Vec`.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub user_authentication: HashSet<AttackPotentialResistance>,

    /// OPTIONAL. A String that contains a URL that links to the certification of the key storage component.
    pub certification: Option<Url>,

    /// OPTIONAL. String that represents a nonce provided by the Issuer to prove that a key attestation was freshly
    /// generated.
    pub nonce: Option<Nonce>,

    /// OPTIONAL. JSON Object representing the supported revocation check mechanisms, such as the one defined in
    /// <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-status-list-12>.
    pub status: Option<StatusClaim>,
    // TODO (PVW-6239): Add `key_storage_status` field to Key Attestation.
}

impl JwtTyp for KeyAttestationClaims {
    const TYP: &'static str = OPENID4VCI_PROOF_JWT_TYP;
}

/// Attack potential resistance of a storage or access method.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.2>
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr)]
pub enum AttackPotentialResistance {
    #[strum(serialize = "iso_18045_high")]
    Iso18045High,

    #[strum(serialize = "iso_18045_moderate")]
    Iso18045Moderate,

    #[strum(serialize = "iso_18045_enhanced-basic")]
    Iso18045EnhancedBasic,

    #[strum(serialize = "iso_18045_basic")]
    Iso18045Basic,

    // Allow the sender to announce an attack potential resistance that the receiver doesn't support.
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Constructor)]
pub struct KeyAttestationVerificationResult {
    pub attested_keys: VecNonEmptyUnique<PublicKey>,
    pub nonce: Option<Nonce>,
    pub status: Option<StatusClaim>,
}

impl KeyAttestation {
    pub async fn sign<'a, I, K, G>(
        attested_keys: I,
        certification: Url,
        nonce: Option<Nonce>,
        key_pair: &KeyPair<K>,
        time: &G,
    ) -> Result<Self, KeyAttestationSigningError>
    where
        I: IntoNonEmptyIterator<Item = &'a PublicKey>,
        K: EcdsaKey,
        G: Generator<DateTime<Utc>>,
    {
        let attested_keys = attested_keys
            .into_nonempty_iter()
            .map(jwk_from_public_key)
            .collect::<Result<_, _>>()
            .map_err(KeyAttestationSigningError::PublicKey)?;

        let claims = KeyAttestationClaims::new(attested_keys, certification, nonce, time);
        let signed_jwt = SignedJwt::sign_with_certificate(&claims, key_pair)
            .await
            .map_err(KeyAttestationSigningError::SignJwt)?;

        Ok(Self(signed_jwt.into()))
    }

    /// Verify the Key Attestation JWT against a list of Trust Anchors. Returns the attested public keys and the nonce,
    /// if present.
    ///
    /// Note that this validates neither the nonce nor revocation using the status list, opting to return both of these
    /// claim values instead.
    pub fn verify<G>(
        &self,
        trust_anchors: &TrustAnchors,
        time: &G,
    ) -> Result<KeyAttestationVerificationResult, KeyAttestationVerificationError>
    where
        G: Generator<DateTime<Utc>>,
    {
        let Self(jwt) = self;

        let (_, claims) = jwt
            .parse_and_verify_against_trust_anchors(
                trust_anchors,
                time,
                Some(CertificateUsage::KeyAttestation),
                ES256_JWT_VALIDATION.clone(),
            )
            .map_err(KeyAttestationVerificationError::InvalidJwt)?;

        let public_keys = claims
            .attested_keys
            .iter()
            .map(jwk_to_public_key)
            .collect::<Result<Vec<_>, _>>()
            .map_err(KeyAttestationVerificationError::InvalidJwk)?;

        let unique_public_keys = public_keys
            .try_into()
            .map_err(|_| KeyAttestationVerificationError::AttestedKeysNotDistinct)?;

        // According to TS3, both the `key_storage` and `user_authentication` fields should be `iso_18045_high`. We
        // assume this implies that they should at least contain this value, disregarding any other values
        // present.
        //
        // Source: <https://github.com/eu-digital-identity-wallet/eudi-doc-standards-and-technical-specifications/blob/5924eb77ab4495d4dc0a874e54ac3e5de1fbd5b1/docs/technical-specifications/ts3-wallet-unit-attestation.md#232-content-of-key-attestation-ka>
        if !claims.key_storage.contains(&AttackPotentialResistance::Iso18045High) {
            return Err(KeyAttestationVerificationError::KeyStorageAttackResistance(
                claims.key_storage,
            ));
        }

        if !claims
            .user_authentication
            .contains(&AttackPotentialResistance::Iso18045High)
        {
            return Err(KeyAttestationVerificationError::UserAuthenticationAttackResistance(
                claims.user_authentication,
            ));
        }

        Ok(KeyAttestationVerificationResult::new(
            unique_public_keys,
            claims.nonce,
            claims.status,
        ))
    }
}

impl KeyAttestationClaims {
    fn new<G>(attested_keys: VecNonEmpty<Jwk>, certification: Url, nonce: Option<Nonce>, time: &G) -> Self
    where
        G: Generator<DateTime<Utc>>,
    {
        let now = time.generate();
        let iat = DateTimeSeconds::new(now);
        let exp = Some(DateTimeSeconds::new(now + VALIDITY_TIME));

        Self {
            iat,
            exp,
            attested_keys,
            // According to TS3, both of these fields should be `iso_18045_high`.
            //
            // Source: <https://github.com/eu-digital-identity-wallet/eudi-doc-standards-and-technical-specifications/blob/5924eb77ab4495d4dc0a874e54ac3e5de1fbd5b1/docs/technical-specifications/ts3-wallet-unit-attestation.md#232-content-of-key-attestation-ka>
            key_storage: HashSet::from([AttackPotentialResistance::Iso18045High]),
            user_authentication: HashSet::from([AttackPotentialResistance::Iso18045High]),
            certification: Some(certification),
            nonce,
            status: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;
    use std::sync::LazyLock;
    use std::time::Duration;

    use chrono::Utc;
    use crypto::PublicKey;
    use crypto::WithVerifyingKey;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use futures::FutureExt;
    use itertools::Itertools;
    use jwt::SignedJwt;
    use jwt::jwk::jwk_from_public_key;
    use jwt::jwk::jwk_to_public_key;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use rstest::rstest;
    use serde_json::json;
    use url::Url;
    use utils::date_time_seconds::DateTimeSeconds;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::NonEmptyIterator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::AttackPotentialResistance;
    use super::KeyAttestation;
    use super::KeyAttestationClaims;
    use super::KeyAttestationVerificationError;
    use super::KeyAttestationVerificationResult;
    use super::VALIDITY_TIME;

    static KEY_PAIR_AND_TRUST_ANCHORS: LazyLock<(KeyPair, TrustAnchors)> = LazyLock::new(|| {
        let ca = Ca::generate_wptl_mock_ca().unwrap();
        let key_pair = ca.generate_key_attestation_mock().unwrap();
        let trust_anchors = TrustAnchors::from(&ca);

        (key_pair, trust_anchors)
    });

    static PUBLIC_KEYS: LazyLock<VecNonEmpty<PublicKey>> = LazyLock::new(|| {
        (0..3)
            .into_iter()
            .map(|_| PublicKey::from(*SigningKey::generate().verifying_key()))
            .collect_vec()
            .try_into()
            .unwrap()
    });

    static CERTIFICATION_URL: LazyLock<Url> =
        LazyLock::new(|| "https://wallet-provider.example.com/certification".parse().unwrap());

    #[test]
    fn test_key_attestation_claims_openid4vci_example_deserialization() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1-6>
        let json = json!({
            "iss": "<identifier of the issuer of this key attestation>",
            "iat": 1516247022,
            "exp": 1541493724,
            "key_storage": [ "iso_18045_moderate" ],
            "user_authentication": [ "iso_18045_moderate" ],
            "attested_keys": [
                {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                    "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            ]
        });

        let claims = serde_json::from_value::<KeyAttestationClaims>(json)
            .expect("example JSON should deserialize to KeyAttestationClaims");

        assert!(claims.exp.is_some_and(|exp| exp.as_ref() > claims.iat.as_ref()));

        let jwk = claims
            .attested_keys
            .iter()
            .exactly_one()
            .expect("claims should contain exactly one attested key");
        let public_key = jwk_to_public_key(jwk).expect("attested key should contain a valid public key");
        assert_matches!(public_key, PublicKey::ESP256(_));

        assert_eq!(
            claims.key_storage,
            HashSet::from([AttackPotentialResistance::Iso18045Moderate])
        );
        assert_eq!(
            claims.user_authentication,
            HashSet::from([AttackPotentialResistance::Iso18045Moderate])
        );
    }

    #[test]
    fn test_key_attestation_claims_ts3_example_deserialization() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1-6>
        let json = json!({
            "iat": 1516247022,
            "exp": 1541493724,
            "certification": "https://example.org/certification/wscd/GlobalPlatform/",
            "key_storage_status": {
                "status": {
                "status_list": {
                    "idx": 7,
                    "uri": "https://revocation_url/wua-type-statuslists/3"
                }
                },
                "exp": 1544172124
            },
            "attested_keys": [
                {
                "kty": "EC",
                "crv": "P-256",
                "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            ],
            "key_storage": [
                "iso_18045_high"
            ],
            "user_authentication": [
                "iso_18045_high",
                "iso_18045_moderate"
            ]
        });

        let claims = serde_json::from_value::<KeyAttestationClaims>(json)
            .expect("example JSON should deserialize to KeyAttestationClaims");

        assert!(claims.exp.is_some_and(|exp| exp.as_ref() > claims.iat.as_ref()));

        assert_eq!(
            claims.certification,
            Some(
                "https://example.org/certification/wscd/GlobalPlatform/"
                    .parse()
                    .unwrap()
            )
        );

        // TODO (PVW-6239): Check `key_storage_status` field contents.

        let jwk = claims
            .attested_keys
            .iter()
            .exactly_one()
            .expect("claims should contain exactly one attested key");
        let public_key = jwk_to_public_key(jwk).expect("attested key should contain a valid public key");
        assert_matches!(public_key, PublicKey::ESP256(_));

        assert_eq!(
            claims.key_storage,
            HashSet::from([AttackPotentialResistance::Iso18045High])
        );
        assert_eq!(
            claims.user_authentication,
            HashSet::from([
                AttackPotentialResistance::Iso18045Moderate,
                AttackPotentialResistance::Iso18045High
            ])
        );
    }

    fn sign_key_attestation(nonce: Option<&str>) -> KeyAttestation {
        let (key_pair, _trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);

        KeyAttestation::sign(
            LazyLock::force(&PUBLIC_KEYS),
            CERTIFICATION_URL.clone(),
            nonce.map(|nonce| nonce.to_string().into()),
            key_pair,
            &MockTimeGenerator::default(),
        )
        .now_or_never()
        .unwrap()
        .unwrap()
    }

    #[rstest]
    fn test_key_attestation_verify_ok(#[values(None, Some("nonce"))] nonce: Option<&str>) {
        let (_key_pair, trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);
        let key_attestation = sign_key_attestation(nonce);

        let KeyAttestationVerificationResult {
            attested_keys: decoded_keys,
            nonce: decoded_nonce,
            ..
        } = key_attestation
            .verify(trust_anchors, &MockTimeGenerator::default())
            .expect("key attestation should verify successfully");

        assert!(decoded_keys.iter().eq(LazyLock::force(&PUBLIC_KEYS)));
        assert_eq!(decoded_nonce.as_ref().map(AsRef::as_ref), nonce);
    }

    #[test]
    fn test_key_attestation_verify_error_invalid_jwt_certificate() {
        let key_attestation = sign_key_attestation(None);

        // Verify a `KeyAttestation` against an empty list of trust anchors.
        let error = key_attestation
            .verify(&TrustAnchors::empty(), &MockTimeGenerator::default())
            .expect_err("key attestation should fail to verify");

        assert_matches!(error, KeyAttestationVerificationError::InvalidJwt(_));
    }

    #[test]
    fn test_key_attestation_verify_error_invalid_jwt_expired() {
        let (key_pair, trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);

        // Create a `KeyAttestation` that was issued yesterday and should be expired.
        let yesterday = Utc::now() - Duration::from_hours(24);
        let key_attestation = KeyAttestation::sign(
            LazyLock::force(&PUBLIC_KEYS),
            CERTIFICATION_URL.clone(),
            None,
            key_pair,
            &MockTimeGenerator::new(yesterday),
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        let error = key_attestation
            .verify(trust_anchors, &MockTimeGenerator::default())
            .expect_err("key attestation should fail to verify");

        assert_matches!(error, KeyAttestationVerificationError::InvalidJwt(_));
    }

    #[test]
    fn test_key_attestation_verify_error_attested_keys_not_distinct() {
        let (key_pair, trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);

        // Create a `KeyAttestation` that has the same public key three times.
        let public_keys = utils::vec_at_least::repeat_n(PUBLIC_KEYS.first().clone(), 3.try_into().unwrap())
            .collect::<VecNonEmpty<_>>();

        let key_attestation = KeyAttestation::sign(
            &public_keys,
            CERTIFICATION_URL.clone(),
            None,
            key_pair,
            &MockTimeGenerator::default(),
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        let error = key_attestation
            .verify(trust_anchors, &MockTimeGenerator::default())
            .expect_err("key attestation should fail to verify");

        assert_matches!(error, KeyAttestationVerificationError::AttestedKeysNotDistinct);
    }

    fn sign_key_attestation_with_attack_resistances(
        key_pair: &KeyPair,
        key_storage: HashSet<AttackPotentialResistance>,
        user_authentication: HashSet<AttackPotentialResistance>,
    ) -> KeyAttestation {
        let now = Utc::now();

        let claims = KeyAttestationClaims {
            iat: DateTimeSeconds::new(now),
            exp: Some(DateTimeSeconds::new(now + VALIDITY_TIME)),
            attested_keys: vec_nonempty![
                jwk_from_public_key(&PublicKey::from(
                    key_pair.verifying_key().now_or_never().unwrap().unwrap()
                ))
                .unwrap()
            ],
            key_storage,
            user_authentication,
            certification: None,
            nonce: None,
            status: None,
        };
        let signed_jwt = SignedJwt::sign_with_certificate(&claims, key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        KeyAttestation(signed_jwt.into())
    }

    #[test]
    fn test_key_attestation_verify_error_key_storage_attack_resistance() {
        let (key_pair, trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);

        // Create a `KeyAttestation` that does not have a key storage attack potential resistance of `iso_18045_high`.
        let key_attestation = sign_key_attestation_with_attack_resistances(
            key_pair,
            HashSet::from([
                AttackPotentialResistance::Iso18045Basic,
                AttackPotentialResistance::Iso18045Moderate,
            ]),
            HashSet::from([AttackPotentialResistance::Iso18045High]),
        );

        let error = key_attestation
            .verify(trust_anchors, &MockTimeGenerator::default())
            .expect_err("key attestation should fail to verify");

        assert_matches!(
            error,
            KeyAttestationVerificationError::KeyStorageAttackResistance(resistance) if resistance == HashSet::from([
                AttackPotentialResistance::Iso18045Basic,
                AttackPotentialResistance::Iso18045Moderate,
            ])
        );
    }

    #[test]
    fn test_key_attestation_verify_error_user_authentication_attack_resistance() {
        let (key_pair, trust_anchors) = &LazyLock::force(&KEY_PAIR_AND_TRUST_ANCHORS);

        // Create a `KeyAttestation` that does not have a user authentication attack potential resistance of
        // `iso_18045_high`.
        let key_attestation = sign_key_attestation_with_attack_resistances(
            key_pair,
            HashSet::from([AttackPotentialResistance::Iso18045High]),
            HashSet::from([AttackPotentialResistance::Iso18045EnhancedBasic]),
        );

        let error = key_attestation
            .verify(trust_anchors, &MockTimeGenerator::default())
            .expect_err("key attestation should fail to verify");

        assert_matches!(
            error,
            KeyAttestationVerificationError::UserAuthenticationAttackResistance(resistance)
                if resistance == HashSet::from([AttackPotentialResistance::Iso18045EnhancedBasic])
        );
    }
}
