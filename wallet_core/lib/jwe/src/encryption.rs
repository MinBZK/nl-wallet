use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use derive_more::Into;
use josekit::JoseError;
use josekit::jwe::JweHeader;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use josekit::jwe::alg::ecdh_es::EcdhEsJweEncrypter;
use jwk_simple::Algorithm;
use jwk_simple::EcCurve;
use jwk_simple::EcParams;
use jwk_simple::Key;
use jwk_simple::KeyParams;
use jwk_simple::KeyType;
use jwk_simple::KeyUse;
use p256::EncodedPoint;
use p256::PublicKey;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::pkcs8::EncodePublicKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;

use crate::algorithm::JweAlgorithm;
use crate::algorithm::JweEncryptionAlgorithm;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum JwePublicKeyError {
    #[error("JWK is not valid: {0}")]
    JwkInvalid(#[source] jwk_simple::Error),

    #[error("JWK does not contain an algorithm")]
    MissingJwkAlgorithm,

    #[error("JWK specifies key use \"{0}\", not encryption")]
    InvalidJwkKeyUse(KeyUse),

    #[error("JWK algorithm \"{0}\" is not supported")]
    UnsupportedJwkAlgorithm(Algorithm),

    #[error("JWK key type \"{0}\" is not consistent with algorithm \"{1}\"")]
    InconsistentJwkKeyType(KeyType, Algorithm),

    #[error("JWK EC curve is \"{0}\", not P-256")]
    UnsupportedJwkEcCurve(EcCurve),
}

#[derive(Debug, Clone, Copy, From, Into, Display, FromStr)]
#[display(
    "{}",
    self
        .0
        .to_public_key_pem(Default::default())
        .expect("a p256 public key should always encode to PKCS #8 PEM")
        .as_str()
)]
struct PemPublicKey(PublicKey);

/// Wraps a P-256 EC public key, anoptional `kid` value and a JWE algorithm. This type is meant to be converted from a
/// JWK in the form of a [`Key`] type and used as key derivation input for encrypting a JWE.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwePublicKey {
    id: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    key: PemPublicKey,
    #[serde_as(as = "DisplayFromStr")]
    algorithm: JweAlgorithm,
}

impl JwePublicKey {
    pub fn new(id: Option<String>, key: PublicKey, algorithm: JweAlgorithm) -> Self {
        Self {
            id,
            key: key.into(),
            algorithm,
        }
    }

    pub fn try_from_jwk(jwk: &Key) -> Result<Self, JwePublicKeyError> {
        jwk.validate().map_err(JwePublicKeyError::JwkInvalid)?;

        let algorithm = jwk.alg().ok_or(JwePublicKeyError::MissingJwkAlgorithm)?;

        if let Some(key_use) = jwk.key_use()
            && *key_use != KeyUse::Encryption
        {
            return Err(JwePublicKeyError::InvalidJwkKeyUse(key_use.clone()));
        }

        let jwe_algorithm = JweAlgorithm::try_from_jwk_simple_algorithm(algorithm)
            .ok_or(JwePublicKeyError::UnsupportedJwkAlgorithm(algorithm.clone()))?;

        if !jwk.is_algorithm_compatible(algorithm) {
            return Err(JwePublicKeyError::InconsistentJwkKeyType(
                jwk.params().key_type(),
                algorithm.clone(),
            ));
        }

        let KeyParams::Ec(ec_params) = jwk.params() else {
            unreachable!(
                "Key::is_algorithm_compatible() in combination with Self::is_algorithm_supported() \
                    guarantees a supported key type"
            );
        };

        if ec_params.crv != EcCurve::P256 {
            return Err(JwePublicKeyError::UnsupportedJwkEcCurve(ec_params.crv));
        }

        let id = jwk.kid().map(str::to_string);

        let encoded_point =
            EncodedPoint::from_affine_coordinates(ec_params.x.as_bytes().into(), ec_params.y.as_bytes().into(), false);
        let public_key = PublicKey::from_encoded_point(&encoded_point)
            .expect("Key::validate() succeeding guarantees valid x and y coordinates");

        Ok(Self::new(id, public_key, jwe_algorithm))
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn key(&self) -> PublicKey {
        self.key.into()
    }

    pub fn algorithm(&self) -> JweAlgorithm {
        self.algorithm
    }

    fn ec_params(&self) -> EcParams {
        let encoded_point = PublicKey::from(self.key).to_encoded_point(false);
        let x = encoded_point
            .x()
            .expect("public key should never use the identity point");
        let y = encoded_point
            .y()
            .expect("public key should never use the identity point");

        EcParams::new_public(EcCurve::P256, x.to_vec().into(), y.to_vec().into())
    }
}

impl From<JwePublicKey> for Key {
    fn from(value: JwePublicKey) -> Self {
        let key = Self::new(KeyParams::Ec(value.ec_params()))
            .with_alg(value.algorithm.into())
            .with_use(KeyUse::Encryption);

        if let Some(id) = value.id { key.with_kid(id) } else { key }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JweEncrypterError {
    #[error("could not serialize data: {0}")]
    Serialization(#[source] serde_json::Error),

    #[error("could not encrypt data: {0}")]
    Encryption(#[source] JoseError),
}

/// Wraps JWE encryption using the key that is derived from an eliptic curve P-256 public key and optional `kid` value.
/// Can be constructed from a [`JwePublicKey`].
#[derive(Debug, Clone)]
pub struct JweEncrypter {
    id: Option<String>,
    encrypter: EcdhEsJweEncrypter,
}

impl JweEncrypter {
    fn new(id: Option<String>, public_key: PublicKey, algorithm: JweAlgorithm) -> Self {
        let der = public_key
            .to_public_key_der()
            .expect("a p256 public key should always encode to PKCS #8 DER");

        let encrypter = EcdhEsJweAlgorithm::from(algorithm)
            .encrypter_from_der(der)
            .expect("the p256 public key PKCS #8 DER should always be valid");

        Self { id, encrypter }
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn encrypt<T>(
        &self,
        data: &T,
        encryption_algorithm: JweEncryptionAlgorithm,
        apu: Option<&[u8]>,
        apv: Option<&[u8]>,
    ) -> Result<String, JweEncrypterError>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(data).map_err(JweEncrypterError::Serialization)?;

        let mut header = JweHeader::new();

        if let Some(id) = &self.id {
            header.set_key_id(id);
        }

        header.set_content_encryption(encryption_algorithm.to_string());

        if let Some(apu) = apu {
            header.set_agreement_partyuinfo(apu);
        }
        if let Some(apv) = apv {
            header.set_agreement_partyvinfo(apv);
        }

        let jwe = josekit::jwe::serialize_compact(&payload, &header, &self.encrypter)
            .map_err(JweEncrypterError::Encryption)?;

        Ok(jwe)
    }
}

impl From<JwePublicKey> for JweEncrypter {
    fn from(value: JwePublicKey) -> Self {
        Self::new(value.id, value.key.into(), value.algorithm)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use jwk_simple::Key;
    use jwk_simple::KeyUse;
    use rstest::rstest;
    use serde::Serialize;
    use serde_json::json;

    use crate::algorithm::JweAlgorithm;
    use crate::algorithm::JweEncryptionAlgorithm;

    use super::JweEncrypter;
    use super::JweEncrypterError;
    use super::JwePublicKey;
    use super::JwePublicKeyErrorDiscriminants;

    fn example_jwk() -> serde_json::Value {
        example_jwk_with_alg(JweAlgorithm::EcdhEs)
    }

    fn example_jwk_with_alg(algorithm: JweAlgorithm) -> serde_json::Value {
        // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.3-6
        json!({
            "kty": "EC",
            "kid": "ac",
            "use": "enc",
            "crv": "P-256",
            "alg": algorithm.to_string(),
            "x": "YO4epjifD-KWeq1sL2tNmm36BhXnkJ0He-WqMYrp9Fk",
            "y": "Hekpm0zfK7C-YccH5iBjcIXgf6YdUvNUac_0At55Okk"
        })
    }

    fn example_jwk_no_kid() -> serde_json::Value {
        let mut json = example_jwk();

        json.as_object_mut().unwrap().remove("kid");

        json
    }

    fn example_jwk_invalid_key_length() -> serde_json::Value {
        let mut p256_json = example_jwk();
        let mut p521_json = example_jwk_p521();

        p256_json
            .as_object_mut()
            .unwrap()
            .insert("x".to_string(), p521_json.as_object_mut().unwrap().remove("x").unwrap());
        p256_json
            .as_object_mut()
            .unwrap()
            .insert("y".to_string(), p521_json.as_object_mut().unwrap().remove("y").unwrap());

        p256_json
    }

    fn example_jwk_no_alg() -> serde_json::Value {
        let mut json = example_jwk();

        json.as_object_mut().unwrap().remove("alg");

        json
    }

    fn example_jwk_key_use_sig() -> serde_json::Value {
        let mut json = example_jwk();

        json.as_object_mut()
            .unwrap()
            .insert("use".to_string(), serde_json::Value::String("sig".to_string()));

        json
    }

    fn example_jwk_alg_es256() -> serde_json::Value {
        let mut json = example_jwk();

        json.as_object_mut()
            .unwrap()
            .insert("alg".to_string(), serde_json::Value::String("ES256".to_string()));

        json
    }

    fn example_jwk_rsa_alg_ecdh_es() -> serde_json::Value {
        json!({
            "kty": "RSA",
            "alg": "ECDH-ES",
            "n": "yeNlzlub94YgerT030codqEztjfU_S6X4DbDA_iVKkjAWtYfPHDzz_sPCT1Axz6isZdf3lHpq_gYX4Sz\
                  -cbe4rjmigxUxr-FgKHQy3HeCdK6hNq9ASQvMK9LBOpXDNn7mei6RZWom4wo3CMvvsY1w8tjtfLb-yQw\
                  JPltHxShZq5-ihC9irpLI9xEBTgG12q5lGIFPhTl_7inA1PFK97LuSLnTJzW0bj096v_TMDg7pOWm_zH\
                  tF53qbVsI0e3v5nmdKXdFf9BjIARRfVrbxVxiZHjU6zL6jY5QJdh1QCmENoejj_ytspMmGW7yMRxzUqg\
                  xcAqOBpVm0b-_mW3HoBdjQ",
            "e": "AQAB"
        })
    }

    fn example_jwk_p521() -> serde_json::Value {
        json!({
            "kty": "EC",
            "crv": "P-521",
            "alg": "ECDH-ES",
            "x": "AekpBQ8ST8a8VcfVOTNl353vSrDCLLJXmPk06wTjxrrjcBpXp5EOnYG_NjFZ6OvLFV1jSfS9tsz4qUxcWceqwQGk",
            "y": "ADSmRA43Z1DSNx_RvcLI87cdL07l6jQyyBXMoxVg_l2Th-x3S1WDhjDly79ajL4Kkd0AZMaZmh9ubmf63e3kyMj2"
        })
    }

    #[rstest]
    #[case::valid(example_jwk(), Ok(()))]
    #[case::valid_ecdh_es_a256kw(example_jwk_with_alg(JweAlgorithm::EcdhEsA256kw), Ok(()))]
    #[case::valid_no_kid(example_jwk_no_kid(), Ok(()))]
    #[case::invalid_key_length(example_jwk_invalid_key_length(), Err(JwePublicKeyErrorDiscriminants::JwkInvalid))]
    #[case::invalid_no_alg(example_jwk_no_alg(), Err(JwePublicKeyErrorDiscriminants::MissingJwkAlgorithm))]
    #[case::invalid_key_use(example_jwk_key_use_sig(), Err(JwePublicKeyErrorDiscriminants::InvalidJwkKeyUse))]
    #[case::invalid_alg_es256(
        example_jwk_alg_es256(),
        Err(JwePublicKeyErrorDiscriminants::UnsupportedJwkAlgorithm)
    )]
    #[case::example_jwk_rsa_alg_ecdh_es(
        example_jwk_rsa_alg_ecdh_es(),
        Err(JwePublicKeyErrorDiscriminants::InconsistentJwkKeyType)
    )]
    #[case::invalid_curve(example_jwk_p521(), Err(JwePublicKeyErrorDiscriminants::UnsupportedJwkEcCurve))]
    fn test_jwe_encryption_key(
        #[case] json: serde_json::Value,
        #[case] expected_result: Result<(), JwePublicKeyErrorDiscriminants>,
    ) {
        let jwk = serde_json::from_value(json).unwrap();
        let result = JwePublicKey::try_from_jwk(&jwk);

        match expected_result {
            Ok(()) => {
                let key = result.expect("converting from JWK to JweEncryptionKey should succeed");

                assert_eq!(key.id(), jwk.kid());

                let output_jwk = Key::from(key);

                assert_eq!(output_jwk.as_ec().unwrap(), jwk.as_ec().unwrap());
                assert_eq!(output_jwk.alg(), jwk.alg());
                assert_eq!(output_jwk.key_use(), Some(&KeyUse::Encryption));
                assert_eq!(output_jwk.kid(), jwk.kid());
            }
            Err(expected_error) => {
                let error = result.expect_err("converting from JWK to JweEncryptionKey should fail");

                assert_eq!(JwePublicKeyErrorDiscriminants::from(&error), expected_error);
            }
        }
    }

    #[derive(Debug, Serialize)]
    struct ValidPayload {
        text: String,
        number: u64,
    }

    impl ValidPayload {
        fn new() -> Self {
            Self {
                text: "Some text".to_string(),
                number: 42,
            }
        }
    }

    #[rstest]
    #[case::ecdh_es(ValidPayload::new(), example_jwk_with_alg(JweAlgorithm::EcdhEs))]
    #[case::ecdh_es_a128kw(ValidPayload::new(), example_jwk_with_alg(JweAlgorithm::EcdhEsA128kw))]
    #[case::ecdh_es_a256kw(ValidPayload::new(), example_jwk_with_alg(JweAlgorithm::EcdhEsA256kw))]
    #[case::no_kid(ValidPayload::new(), example_jwk_no_kid())]
    fn test_jwe_encrypter_ok<T>(
        #[case] data: T,
        #[case] jwk_json: serde_json::Value,
        #[values(
            JweEncryptionAlgorithm::A128CbcHs256,
            JweEncryptionAlgorithm::A256CbcHs512,
            JweEncryptionAlgorithm::A128Gcm,
            JweEncryptionAlgorithm::A256Gcm
        )]
        encryption_algorithm: JweEncryptionAlgorithm,
        #[values(None, Some("apu"))] apu: Option<&str>,
        #[values(None, Some("apv"))] apv: Option<&str>,
    ) where
        T: Serialize,
    {
        let jwk = serde_json::from_value(jwk_json).unwrap();
        let key = JwePublicKey::try_from_jwk(&jwk).unwrap();
        let encrypter = JweEncrypter::from(key);

        let jws = encrypter
            .encrypt(
                &data,
                encryption_algorithm,
                apu.map(str::as_bytes),
                apv.map(str::as_bytes),
            )
            .expect("encrypting data with JweEncrypter should succeed");

        let header = josekit::jwt::decode_header(&jws).unwrap();

        assert_eq!(
            header.claim("alg").and_then(serde_json::Value::as_str),
            Some(jwk.alg().unwrap().to_string().as_str())
        );
        assert_eq!(
            header.claim("enc").and_then(serde_json::Value::as_str),
            Some(encryption_algorithm.to_string().as_str())
        );
        assert!(header.claim("epk").and_then(serde_json::Value::as_object).is_some());

        assert_eq!(header.claim("kid").and_then(serde_json::Value::as_str), jwk.kid());
        assert_eq!(header.claim("apu").is_some(), apu.is_some());
        assert_eq!(header.claim("apv").is_some(), apv.is_some());
    }

    #[test]
    fn test_jwe_encrypter_error_serialization() {
        let jwk = serde_json::from_value(example_jwk()).unwrap();
        let key = JwePublicKey::try_from_jwk(&jwk).unwrap();
        let encrypter = JweEncrypter::from(key);

        let data = HashMap::from([(("one".to_string(), "two".to_string()), "three".to_string())]);
        let result = encrypter.encrypt(&data, JweEncryptionAlgorithm::A256Gcm, None, None);

        let error = result.expect_err("encrypting data with JweEncrypter should fail");

        assert_matches!(error, JweEncrypterError::Serialization(_));
    }
}
