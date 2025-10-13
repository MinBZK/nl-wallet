use base64::prelude::*;
use rand::Rng;
use serde_json::Value;

use attestation_types::claim_path::ClaimPath;
use crypto::utils::random_bytes;
use utils::vec_at_least::VecNonEmpty;

use crate::claims::ClaimValue;
use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::disclosure::DisclosureContentSerializationError;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::sd_jwt::SdJwtVcClaims;

pub(crate) const DEFAULT_SALT_SIZE: usize = 30;

/// Transforms a JSON object into an SD-JWT object by substituting selected values
/// with their corresponding disclosure digests.
#[derive(Debug, Clone)]
pub struct SdObjectEncoder<H> {
    /// The object in JSON format.
    object: SdJwtVcClaims,
    /// Size of random data used to generate the salts for disclosures in bytes.
    /// Constant length for readability considerations.
    salt_size: usize,
    /// The hash function used to create digests.
    pub(crate) hasher: H,
}

impl TryFrom<Value> for SdObjectEncoder<Sha256Hasher> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Self::with_custom_hasher_and_salt_size(value, Sha256Hasher, DEFAULT_SALT_SIZE)
    }
}

impl<H: Hasher> SdObjectEncoder<H> {
    /// Creates a new [`SdObjectEncoder`] with custom hash function to create digests, and custom salt size.
    pub fn with_custom_hasher_and_salt_size(object: Value, hasher: H, salt_size: usize) -> Result<Self> {
        if !object.is_object() {
            return Err(Error::DataTypeMismatch(
                "argument `object` must be a JSON Object".to_string(),
            ));
        };

        Ok(Self {
            object: serde_json::from_value(object)?,
            salt_size,
            hasher,
        })
    }

    pub fn encode(mut self) -> SdJwtVcClaims {
        self.object._sd_alg = Some(self.hasher.alg());
        self.object
    }

    /// Substitutes a value with the digest of its disclosure.
    ///
    /// `path` indicates the claim paths pointing to the value that will be concealed.
    pub fn conceal(&mut self, path: VecNonEmpty<ClaimPath>) -> Result<Disclosure> {
        // Determine salt.
        let salt = Self::gen_rand(self.salt_size);

        let (rest, last_path) = path.into_inner_last();
        let parent = self.object.claims.traverse_by_claim_paths(rest.iter())?;

        match parent {
            Some(claim) => claim.conceal(&last_path, salt, &self.hasher),
            None => Err(Error::ParentNotFound(rest)),
        }
    }

    /// Adds a decoy digest to the specified path.
    ///
    /// `path` indicates the pointer to the value that will be concealed using the syntax of
    /// [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901).
    ///
    /// Use `path` = "" to add decoys to the top level.
    pub fn add_decoys(&mut self, path: &[ClaimPath], number_of_decoys: usize) -> Result<()> {
        for _ in 0..number_of_decoys {
            self.add_decoy(path)?;
        }
        Ok(())
    }

    fn add_decoy(&mut self, path: &[ClaimPath]) -> Result<()> {
        self.object.claims.add_decoy(path, &self.hasher, self.salt_size)
    }

    pub(crate) fn random_digest(hasher: &H, salt_len: usize, array_entry: bool) -> Result<String> {
        let mut rng = rand::thread_rng();
        let salt = Self::gen_rand(salt_len);
        let decoy_value_length = rng.gen_range(20..=100);
        let decoy_claim_name = if array_entry {
            None
        } else {
            let decoy_claim_name_length = rng.gen_range(4..=10);
            Some(Self::gen_rand(decoy_claim_name_length))
        };
        let decoy_value = Self::gen_rand(decoy_value_length);
        let disclosure = Disclosure::try_new(match decoy_claim_name {
            Some(claim_name) => {
                DisclosureContent::ObjectProperty(salt, claim_name.parse()?, ClaimValue::String(decoy_value))
            }
            None => DisclosureContent::ArrayElement(salt, ClaimValue::String(decoy_value).into()),
        })
        .map_err(|DisclosureContentSerializationError { error, .. }| error)?;
        Ok(hasher.encoded_digest(disclosure.encoded()))
    }

    fn gen_rand(len: usize) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(random_bytes(len))
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use assert_matches::assert_matches;
    use serde_json::Value;
    use serde_json::json;

    use attestation_types::claim_path::ClaimPath;

    use crate::claims::ClaimName;
    use crate::claims::ObjectClaims;
    use crate::error::Error;

    use super::SdObjectEncoder;

    impl<H> SdObjectEncoder<H> {
        pub fn object_claims(&self) -> &ObjectClaims {
            self.object.claims()
        }
    }

    fn object() -> Value {
        json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "id": "did:value",
            "claim1": {
                "abc": true
            },
            "claim2": ["arr-value1", "arr-value2"],
            "vct": "com.example.pid",
            "cnf": {
                "jwk": {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                    "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            }
        })
    }

    #[test]
    fn simple() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();
        encoder
            .conceal(
                vec![
                    ClaimPath::SelectByKey(String::from("claim1")),
                    ClaimPath::SelectByKey(String::from("abc")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap();
        encoder
            .conceal(vec![ClaimPath::SelectByKey(String::from("id"))].try_into().unwrap())
            .unwrap();
        encoder.add_decoys(&[], 10).unwrap();
        encoder
            .add_decoys(&[ClaimPath::SelectByKey(String::from("claim2"))], 10)
            .unwrap();
        assert!(
            !encoder
                .object
                .claims()
                .claims
                .contains_key(&"id".parse::<ClaimName>().unwrap())
        );
        assert_eq!(
            encoder.object.claims()._sd.as_ref().unwrap().len(),
            11.try_into().unwrap()
        );
        assert_eq!(
            encoder
                .object
                .claims()
                .claims
                .get(&"claim2".parse::<ClaimName>().unwrap())
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            12
        );
    }

    #[test]
    fn nested() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();

        encoder
            .conceal(
                vec![
                    ClaimPath::SelectByKey(String::from("claim1")),
                    ClaimPath::SelectByKey(String::from("abc")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap();

        encoder
            .conceal(vec![ClaimPath::SelectByKey(String::from("claim1"))].try_into().unwrap())
            .unwrap();

        assert!(
            !encoder
                .object
                .claims()
                .claims
                .contains_key(&"claim1".parse::<ClaimName>().unwrap())
        );
        assert_eq!(
            encoder.object.claims()._sd.as_ref().unwrap().len(),
            1.try_into().unwrap()
        );
    }

    #[test]
    fn errors() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();
        encoder
            .conceal(
                vec![
                    ClaimPath::SelectByKey(String::from("claim1")),
                    ClaimPath::SelectByKey(String::from("abc")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap();
        assert_matches!(
            encoder
                .conceal(
                    vec![
                        ClaimPath::SelectByKey(String::from("claim2")),
                        ClaimPath::SelectByIndex(2),
                    ]
                    .try_into()
                    .unwrap(),
                )
                .unwrap_err(),
            Error::IndexOutOfBounds(2, _)
        );
    }

    #[test]
    fn test_wrong_path() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();
        assert_matches!(
            encoder
                .conceal(
                    vec![ClaimPath::SelectByKey(String::from("claim12"))]
                        .try_into()
                        .unwrap()
                )
                .unwrap_err(),
            Error::ObjectFieldNotFound(key, _) if key == "claim12".parse().unwrap()
        );
        assert_matches!(
            encoder
                .conceal(
                    vec![
                        ClaimPath::SelectByKey(String::from("claim12")),
                        ClaimPath::SelectByIndex(0),
                    ]
                    .try_into()
                    .unwrap(),
                )
                .unwrap_err(),
            Error::ParentNotFound(_)
        );
    }
}
