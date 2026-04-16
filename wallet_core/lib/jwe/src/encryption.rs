use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use derive_more::Into;
use josekit::jwe::JweHeader;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use josekit::jwe::alg::ecdh_es::EcdhEsJweEncrypter;
use jwk_simple::EcCurve;
use jwk_simple::EcParams;
use jwk_simple::Key;
use jwk_simple::KeyParams;
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

use crate::algorithm::EcdhAlgorithm;
use crate::algorithm::EncryptionAlgorithm;
use crate::error::EcdhPublicJwkError;
use crate::error::JweJsonEncryptionError;
use crate::error::JwkError;

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

/// Wraps a P-256 EC public key, an optional `kid` value and a JWE algorithm. This type is meant to be converted from a
/// JWK in the form of a [`Key`] type and used as key derivation input for encrypting a JWE.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwePublicKey {
    id: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    key: PemPublicKey,
    #[serde_as(as = "DisplayFromStr")]
    algorithm: EcdhAlgorithm,
}

impl JwePublicKey {
    pub fn new(id: Option<String>, key: PublicKey, algorithm: EcdhAlgorithm) -> Self {
        Self {
            id,
            key: key.into(),
            algorithm,
        }
    }

    pub fn try_from_jwk(jwk: &Key) -> Result<Self, EcdhPublicJwkError> {
        jwk.validate().map_err(JwkError::Invalid)?;

        let algorithm = jwk.alg().ok_or(EcdhPublicJwkError::MissingAlgorithm)?;

        if let Some(key_use) = jwk.key_use()
            && *key_use != KeyUse::Encryption
        {
            return Err(JwkError::InvalidKeyUse(key_use.clone()).into());
        }

        let jwe_algorithm = EcdhAlgorithm::try_from_jwk_simple_algorithm(algorithm)
            .ok_or(JwkError::UnsupportedAlgorithm(algorithm.clone()))?;

        if !jwk.is_algorithm_compatible(algorithm) {
            return Err(JwkError::InconsistentKeyType(jwk.params().key_type(), algorithm.clone()).into());
        }

        let KeyParams::Ec(ec_params) = jwk.params() else {
            unreachable!(
                "Key::is_algorithm_compatible() in combination with EcdhAlgorithm::try_from_jwk_simple_algorithm() \
                 guarantees a supported key type"
            );
        };

        if ec_params.crv != EcCurve::P256 {
            return Err(EcdhPublicJwkError::UnsupportedJwkEcCurve(ec_params.crv));
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

    pub fn algorithm(&self) -> EcdhAlgorithm {
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

// Even though the conversion does not require an owned `Key`, this
// trait implementation is useful for `serde_with::TryFrom_Into`.
impl TryFrom<Key> for JwePublicKey {
    type Error = EcdhPublicJwkError;

    fn try_from(value: Key) -> Result<Self, Self::Error> {
        Self::try_from_jwk(&value)
    }
}

/// Wraps JWE encryption using the key that is derived from an eliptic curve P-256 public key and optional `kid` value.
/// Can be constructed from a [`JwePublicKey`].
#[derive(Debug, Clone)]
pub struct JweEncrypter {
    id: Option<String>,
    encrypter: EcdhEsJweEncrypter,
}

#[derive(Debug, Clone, Copy)]
pub enum JweCompression {
    None,
    Deflate,
}

impl JweEncrypter {
    fn new(id: Option<String>, public_key: PublicKey, algorithm: EcdhAlgorithm) -> Self {
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

    pub fn encrypt_json<T>(
        &self,
        data: &T,
        encryption_algorithm: EncryptionAlgorithm,
        apu: Option<&[u8]>,
        apv: Option<&[u8]>,
        use_compression: JweCompression,
    ) -> Result<String, JweJsonEncryptionError>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(data).map_err(JweJsonEncryptionError::Serialization)?;

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

        if matches!(use_compression, JweCompression::Deflate) {
            header.set_compression("DEF");
        }

        let jwe = josekit::jwe::serialize_compact(&payload, &header, &self.encrypter)
            .map_err(JweJsonEncryptionError::Encryption)?;

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

    use crate::algorithm::EcdhAlgorithm;
    use crate::algorithm::EncryptionAlgorithm;
    use crate::error::EcdhPublicJwkError;
    use crate::error::EcdhPublicJwkErrorDiscriminants;
    use crate::error::JwkErrorDiscriminants;

    use super::JweCompression;
    use super::JweEncrypter;
    use super::JweJsonEncryptionError;
    use super::JwePublicKey;

    fn example_jwk() -> serde_json::Value {
        example_jwk_with_alg(EcdhAlgorithm::EcdhEs)
    }

    fn example_jwk_with_alg(algorithm: EcdhAlgorithm) -> serde_json::Value {
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

    #[derive(Clone, Copy)]
    enum ExpectedTryFromJwkError {
        Jwk(JwkErrorDiscriminants),
        EcdhPublicJwk(EcdhPublicJwkErrorDiscriminants),
    }

    #[rstest]
    #[case::valid(example_jwk(), None)]
    #[case::valid_ecdh_es_a256kw(example_jwk_with_alg(EcdhAlgorithm::EcdhEsA256kw), None)]
    #[case::valid_no_kid(example_jwk_no_kid(), None)]
    #[case::invalid_key_length(
        example_jwk_invalid_key_length(),
        Some(ExpectedTryFromJwkError::Jwk(JwkErrorDiscriminants::Invalid))
    )]
    #[case::invalid_no_alg(
        example_jwk_no_alg(),
        Some(ExpectedTryFromJwkError::EcdhPublicJwk(EcdhPublicJwkErrorDiscriminants::MissingAlgorithm))
    )]
    #[case::invalid_key_use(
        example_jwk_key_use_sig(),
        Some(ExpectedTryFromJwkError::Jwk(JwkErrorDiscriminants::InvalidKeyUse))
    )]
    #[case::invalid_alg_es256(
        example_jwk_alg_es256(),
        Some(ExpectedTryFromJwkError::Jwk(JwkErrorDiscriminants::UnsupportedAlgorithm))
    )]
    #[case::example_jwk_rsa_alg_ecdh_es(
        example_jwk_rsa_alg_ecdh_es(),
        Some(ExpectedTryFromJwkError::Jwk(JwkErrorDiscriminants::InconsistentKeyType))
    )]
    #[case::invalid_curve(
        example_jwk_p521(),
        Some(ExpectedTryFromJwkError::EcdhPublicJwk(EcdhPublicJwkErrorDiscriminants::UnsupportedJwkEcCurve))
    )]
    fn test_jwe_encryption_key(
        #[case] json: serde_json::Value,
        #[case] expected_error: Option<ExpectedTryFromJwkError>,
    ) {
        let jwk = serde_json::from_value(json).unwrap();
        let result = JwePublicKey::try_from_jwk(&jwk);

        match expected_error {
            None => {
                let key = result.expect("converting from JWK to JweEncryptionKey should succeed");

                assert_eq!(key.id(), jwk.kid());

                let output_jwk = Key::from(key);

                assert_eq!(output_jwk.as_ec().unwrap(), jwk.as_ec().unwrap());
                assert_eq!(output_jwk.alg(), jwk.alg());
                assert_eq!(output_jwk.key_use(), Some(&KeyUse::Encryption));
                assert_eq!(output_jwk.kid(), jwk.kid());
            }
            Some(expected_error) => {
                let error = result.expect_err("converting from JWK to JweEncryptionKey should fail");

                match expected_error {
                    ExpectedTryFromJwkError::Jwk(expected_discriminant) => {
                        assert_matches!(
                            error,
                            EcdhPublicJwkError::Jwk(error)
                                if JwkErrorDiscriminants::from(&error) == expected_discriminant
                        )
                    }
                    ExpectedTryFromJwkError::EcdhPublicJwk(expected_discriminant) => {
                        assert_eq!(EcdhPublicJwkErrorDiscriminants::from(&error), expected_discriminant)
                    }
                }
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
    #[case::ecdh_es(ValidPayload::new(), example_jwk_with_alg(EcdhAlgorithm::EcdhEs))]
    #[case::ecdh_es_a128kw(ValidPayload::new(), example_jwk_with_alg(EcdhAlgorithm::EcdhEsA128kw))]
    #[case::ecdh_es_a192kw(ValidPayload::new(), example_jwk_with_alg(EcdhAlgorithm::EcdhEsA192kw))]
    #[case::ecdh_es_a256kw(ValidPayload::new(), example_jwk_with_alg(EcdhAlgorithm::EcdhEsA256kw))]
    #[case::no_kid(ValidPayload::new(), example_jwk_no_kid())]
    fn test_jwe_encrypter_ok<T>(
        #[case] data: T,
        #[case] jwk_json: serde_json::Value,
        #[values(
            EncryptionAlgorithm::A128CbcHs256,
            EncryptionAlgorithm::A256CbcHs512,
            EncryptionAlgorithm::A128Gcm,
            EncryptionAlgorithm::A256Gcm
        )]
        encryption_algorithm: EncryptionAlgorithm,
        #[values(None, Some("apu"))] apu: Option<&str>,
        #[values(None, Some("apv"))] apv: Option<&str>,
        #[values(JweCompression::None, JweCompression::Deflate)] use_compression: JweCompression,
    ) where
        T: Serialize,
    {
        let jwk = serde_json::from_value(jwk_json).unwrap();
        let key = JwePublicKey::try_from_jwk(&jwk).unwrap();
        let encrypter = JweEncrypter::from(key);

        let jws = encrypter
            .encrypt_json(
                &data,
                encryption_algorithm,
                apu.map(str::as_bytes),
                apv.map(str::as_bytes),
                use_compression,
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

        if matches!(use_compression, JweCompression::Deflate) {
            assert_eq!(header.claim("zip").and_then(serde_json::Value::as_str), Some("DEF"));
        } else {
            assert!(header.claim("zip").is_none())
        }
    }

    #[test]
    fn test_jwe_encrypter_error_serialization() {
        let jwk = serde_json::from_value(example_jwk()).unwrap();
        let key = JwePublicKey::try_from_jwk(&jwk).unwrap();
        let encrypter = JweEncrypter::from(key);

        let data = HashMap::from([(("one".to_string(), "two".to_string()), "three".to_string())]);
        let result = encrypter.encrypt_json(&data, EncryptionAlgorithm::A256Gcm, None, None, JweCompression::None);

        let error = result.expect_err("encrypting data with JweEncrypter should fail");

        assert_matches!(error, JweJsonEncryptionError::Serialization(_));
    }
}
