// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use base64::prelude::*;
use json_pointer::JsonPointer;
use rand::Rng;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;

use crypto::utils::random_bytes;

use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;

pub(crate) const DIGESTS_KEY: &str = "_sd";
pub(crate) const ARRAY_DIGEST_KEY: &str = "...";
pub(crate) const DEFAULT_SALT_SIZE: usize = 30;
pub(crate) const SD_ALG: &str = "_sd_alg";

/// Transforms a JSON object into an SD-JWT object by substituting selected values
/// with their corresponding disclosure digests.
#[derive(Debug, Clone)]
pub struct SdObjectEncoder<H> {
    /// The object in JSON format.
    object: Value,
    /// Size of random data used to generate the salts for disclosures in bytes.
    /// Constant length for readability considerations.
    salt_size: usize,
    /// The hash function used to create digests.
    pub(crate) hasher: H,
}

impl TryFrom<Value> for SdObjectEncoder<Sha256Hasher> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Self::with_custom_hasher_and_salt_size(value, Sha256Hasher::new(), DEFAULT_SALT_SIZE)
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
            object,
            salt_size,
            hasher,
        })
    }

    pub fn encode(self) -> Value {
        self.object
    }

    /// Substitutes a value with the digest of its disclosure.
    ///
    /// `path` indicates the pointer to the value that will be concealed using the syntax of
    /// [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901).
    ///
    /// ## Error
    /// * [`Error::InvalidPath`] if pointer is invalid.
    /// * [`Error::DataTypeMismatch`] if existing SD format is invalid.
    pub fn conceal(&mut self, path: &str) -> Result<Disclosure> {
        // Determine salt.
        let salt = Self::gen_rand(self.salt_size);

        let element_pointer = path
            .parse::<JsonPointer<_, _>>()
            .map_err(|_| Error::InvalidPath(path.to_string()))?;

        let mut parent_pointer = element_pointer.clone();
        let element_key = parent_pointer.pop().ok_or(Error::InvalidPath(path.to_string()))?;

        let parent = parent_pointer
            .get(&self.object)
            .map_err(|_| Error::InvalidPath(path.to_string()))?;

        match parent {
            Value::Object(_) => {
                let parent = parent_pointer
                    .get_mut(&mut self.object)
                    .map_err(|_| Error::InvalidPath(path.to_string()))?
                    .as_object_mut()
                    .ok_or_else(|| Error::InvalidPath(path.to_string()))?;

                // Remove the value from the parent and create a disclosure for it.
                let disclosure = Disclosure::try_new(DisclosureContent::ObjectProperty(
                    salt,
                    element_key.to_owned(),
                    parent
                        .remove(&element_key)
                        .ok_or_else(|| Error::InvalidPath(path.to_string()))?,
                ))?;

                // Hash the disclosure.
                let hash = self.hasher.encoded_digest(disclosure.as_str());

                // Add the hash to the "_sd" array if exists; otherwise, create the array and insert the hash.
                Self::add_digest_to_object(parent, hash)?;
                Ok(disclosure)
            }
            Value::Array(_) => {
                let element = element_pointer
                    .get_mut(&mut self.object)
                    .map_err(|_| Error::InvalidPath(path.to_string()))?;
                let disclosure = Disclosure::try_new(DisclosureContent::ArrayElement(salt, element.clone()))?;
                let hash = self.hasher.encoded_digest(disclosure.as_str());
                let tripledot = json!({ARRAY_DIGEST_KEY: hash});
                *element = tripledot;
                Ok(disclosure)
            }
            _ => Err(Error::Unspecified(
                "parent of element can can only be an object or an array".to_string(),
            )),
        }
    }

    /// Adds the `_sd_alg` property to the top level of the object.
    /// The value is taken from the [`crate::Hasher::alg_name`] implementation.
    pub fn add_sd_alg_property(&mut self) {
        self.object
            .as_object_mut()
            // Safety: `object` is a JSON object.
            .unwrap()
            .insert(SD_ALG.to_string(), Value::String(self.hasher.alg_name().to_string()));
    }

    /// Adds a decoy digest to the specified path.
    ///
    /// `path` indicates the pointer to the value that will be concealed using the syntax of
    /// [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901).
    ///
    /// Use `path` = "" to add decoys to the top level.
    pub fn add_decoys(&mut self, path: &str, number_of_decoys: usize) -> Result<()> {
        for _ in 0..number_of_decoys {
            self.add_decoy(path)?;
        }
        Ok(())
    }

    fn add_decoy(&mut self, path: &str) -> Result<()> {
        let element_pointer = path
            .parse::<JsonPointer<_, _>>()
            .map_err(|_| Error::InvalidPath(path.to_string()))?;

        let value = element_pointer
            .get_mut(&mut self.object)
            .map_err(|_| Error::InvalidPath(path.to_string()))?;

        if let Some(object) = value.as_object_mut() {
            let hash = Self::random_digest(&self.hasher, self.salt_size, false)?;
            Self::add_digest_to_object(object, hash)?;
            Ok(())
        } else if let Some(array) = value.as_array_mut() {
            let hash = Self::random_digest(&self.hasher, self.salt_size, true)?;
            let tripledot = json!({ARRAY_DIGEST_KEY: hash});
            array.push(tripledot);
            Ok(())
        } else {
            Err(Error::InvalidPath(path.to_string()))
        }
    }

    /// Add the hash to the "_sd" array if exists; otherwise, create the array and insert the hash.
    fn add_digest_to_object(object: &mut Map<String, Value>, digest: String) -> Result<()> {
        if let Some(sd_value) = object.get_mut(DIGESTS_KEY) {
            if let Value::Array(value) = sd_value {
                // Make sure the digests are sorted.
                let idx = value
                    .iter()
                    .enumerate()
                    .find(|(_, value)| value.as_str().is_some_and(|s| s > digest.as_str()))
                    .map(|(pos, _)| pos)
                    .unwrap_or(value.len());
                value.insert(idx, Value::String(digest));
            } else {
                return Err(Error::DataTypeMismatch(
                    "invalid object: existing `_sd` type is not an array".to_string(),
                ));
            }
        } else {
            object.insert(DIGESTS_KEY.to_owned(), json!([digest]));
        }
        Ok(())
    }

    fn random_digest(hasher: &dyn Hasher, salt_len: usize, array_entry: bool) -> Result<String> {
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
            Some(claim_name) => DisclosureContent::ObjectProperty(salt, claim_name, Value::String(decoy_value)),
            None => DisclosureContent::ArrayElement(salt, Value::String(decoy_value)),
        })?;
        Ok(hasher.encoded_digest(disclosure.as_str()))
    }

    fn gen_rand(len: usize) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(random_bytes(len))
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use serde_json::Value;

    use crate::error::Error;

    use super::SdObjectEncoder;

    fn object() -> Value {
        json!({
          "id": "did:value",
          "claim1": {
            "abc": true
          },
          "claim2": ["arr-value1", "arr-value2"]
        })
    }

    #[test]
    fn simple() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();
        encoder.conceal("/claim1/abc").unwrap();
        encoder.conceal("/id").unwrap();
        encoder.add_decoys("", 10).unwrap();
        encoder.add_decoys("/claim2", 10).unwrap();
        assert!(encoder.object.get("id").is_none());
        assert_eq!(encoder.object.get("_sd").unwrap().as_array().unwrap().len(), 11);
        assert_eq!(encoder.object.get("claim2").unwrap().as_array().unwrap().len(), 12);
    }

    #[test]
    fn test_wrong_path() {
        let mut encoder = SdObjectEncoder::try_from(object()).unwrap();
        assert!(matches!(
            encoder.conceal("/claim12").unwrap_err(),
            Error::InvalidPath(_)
        ));
        assert!(matches!(
            encoder.conceal("/claim12/0").unwrap_err(),
            Error::InvalidPath(_)
        ));
    }
}
