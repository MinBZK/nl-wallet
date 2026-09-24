use std::collections::HashSet;

use attestation_types::status_claim::StatusClaim;
use jwt::JwtTyp;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithTyp;
use jwt::headers::HeaderWithX5c;
use jwt::jwk::Jwk;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::skip_serializing_none;
use strum::EnumString;
use url::Url;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

pub const OPENID4VCI_PROOF_JWT_TYP: &str = "key-attestation+jwt";

pub type KeyAttestation = UnverifiedJwt<KeyAttestationClaims, KeyAttestationHeader>;

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

/// The JWT claims for an OpenID4VCI Key Attestation, to be used by the wallet as proof of cryptographic key material in
/// a Credential Request.
///
/// Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttestationClaims {
    /// Integer for the time at which the key attestation was issued using the syntax defined in RFC7519.
    pub iat: DateTimeSeconds,

    /// Integer for the time at which the key attestation and the key(s) it is attesting expire, using the syntax
    /// defined in RFC7519. MUST be present if the attestation is used with the JWT proof type.
    pub exp: Option<DateTimeSeconds>,

    /// A non-empty array of attested keys from the same key storage component using the syntax of JWK as defined in
    /// RFC7517.
    pub attested_keys: VecNonEmpty<Jwk>,

    /// A non-empty array of case sensitive strings that assert the attack potential resistance of the key storage
    /// component and its keys attested in the attested_keys parameter.
    ///
    /// Note that this uses a `HashSet`, which makes more sense semantically than a `Vec`.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub key_storage: HashSet<AttackPotentialResistance>,

    /// A non-empty array of case sensitive strings that assert the attack potential resistance of the user
    /// authentication methods allowed to access the private keys from the attested_keys parameter.
    ///
    /// Note that this uses a `HashSet`, which makes more sense semantically than a `Vec`.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub user_authentication: HashSet<AttackPotentialResistance>,

    /// A String that contains a URL that links to the certification of the key storage component.
    pub certification: Option<Url>,

    /// String that represents a nonce provided by the Issuer to prove that a key attestation was freshly generated.
    pub nonce: Option<Nonce>,

    /// JSON Object representing the supported revocation check mechanisms, such as the one defined in
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

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;

    use crypto::PublicKey;
    use itertools::Itertools;
    use jwt::jwk::jwk_to_public_key;
    use serde_json::json;

    use super::AttackPotentialResistance;
    use super::KeyAttestationClaims;

    #[test]
    fn test_key_attestation_claims_example_deserialization() {
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
}
