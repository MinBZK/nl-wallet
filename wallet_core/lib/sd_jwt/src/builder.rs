// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::VerifyingKey;
use serde::Serialize;

use crypto::EcdsaKeySend;
use jwt::jwk::jwk_from_p256;
use jwt::VerifiedJwt;

use crate::disclosure::Disclosure;
use crate::encoder::SdObjectEncoder;
use crate::encoder::DEFAULT_SALT_SIZE;
use crate::error::Error;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::key_binding_jwt_claims::RequiredKeyBinding;
use crate::sd_jwt::SdJwt;
use crate::sd_jwt::SdJwtClaims;

const SD_JWT_HEADER_TYP: &str = "dc+sd-jwt";

/// Builder structure to create an issuable SD-JWT.
#[derive(Debug)]
pub struct SdJwtBuilder<H> {
    encoder: SdObjectEncoder<H>,
    header: Header,
    disclosures: Vec<Disclosure>,
    key_bind: Option<RequiredKeyBinding>,
}

impl SdJwtBuilder<Sha256Hasher> {
    /// Creates a new [`SdJwtBuilder`] with `sha-256` hash function.
    ///
    /// ## Error
    /// Returns [`Error::DataTypeMismatch`] if `object` is not a valid JSON object.
    pub fn new<T: Serialize>(object: T) -> Result<Self> {
        Self::new_with_hasher(object, Sha256Hasher::new())
    }
}

impl<H: Hasher> SdJwtBuilder<H> {
    /// Creates a new [`SdJwtBuilder`] with custom hash function to create digests.
    pub fn new_with_hasher<T: Serialize>(object: T, hasher: H) -> Result<Self> {
        Self::new_with_hasher_and_salt_size(object, hasher, DEFAULT_SALT_SIZE)
    }

    /// Creates a new [`SdJwtBuilder`] with custom hash function to create digests, and custom salt size.
    pub fn new_with_hasher_and_salt_size<T: Serialize>(object: T, hasher: H, salt_size: usize) -> Result<Self> {
        let object = serde_json::to_value(object)?;
        let encoder = SdObjectEncoder::with_custom_hasher_and_salt_size(object, hasher, salt_size)?;
        Ok(Self {
            encoder,
            disclosures: vec![],
            key_bind: None,
            header: Header {
                typ: Some(String::from(SD_JWT_HEADER_TYP)),
                ..Default::default()
            },
        })
    }

    /// Substitutes a value with the digest of its disclosure.
    ///
    /// ## Notes
    /// - `path` indicates the pointer to the value that will be concealed using the syntax of [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901).
    ///
    ///
    /// ## Example
    ///  ```rust
    ///  use sd_jwt::builder::SdJwtBuilder;
    ///  use serde_json::json;
    ///
    ///  let obj = json!({
    ///   "id": "did:value",
    ///   "claim1": {
    ///      "abc": true
    ///   },
    ///   "claim2": ["val_1", "val_2"]
    /// });
    /// let builder = SdJwtBuilder::new(obj)
    ///   .unwrap()
    ///   .make_concealable("/id").unwrap() //conceals "id": "did:value"
    ///   .make_concealable("/claim1/abc").unwrap() //"abc": true
    ///   .make_concealable("/claim2/0").unwrap(); //conceals "val_1"
    /// ```
    /// ## Error
    /// * [`Error::InvalidPath`] if pointer is invalid.
    /// * [`Error::DataTypeMismatch`] if existing SD format is invalid.
    pub fn make_concealable(mut self, path: &str) -> Result<Self> {
        let disclosure = self.encoder.conceal(path)?;
        self.disclosures.push(disclosure);

        Ok(self)
    }

    /// Adds a new claim to the underlying object.
    pub fn insert_claim<'a, K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: Into<Cow<'a, str>>,
        V: Serialize,
    {
        let key = key.into().into_owned();
        let value = serde_json::to_value(value)?;
        self.encoder
            .object
            .as_object_mut()
            .expect("encoder::object is a JSON Object")
            .insert(key, value);

        Ok(self)
    }

    /// Adds a decoy digest to the specified path.
    ///
    /// `path` indicates the pointer to the value that will be concealed using the syntax of
    /// [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901).
    ///
    /// Use `path` = "" to add decoys to the top level.
    pub fn add_decoys(mut self, path: &str, number_of_decoys: usize) -> Result<Self> {
        self.encoder.add_decoys(path, number_of_decoys)?;

        Ok(self)
    }

    /// Require a proof of possession of a given key from the holder.
    ///
    /// This operation adds a JWT confirmation (`cnf`) claim as specified in
    /// [RFC8300](https://www.rfc-editor.org/rfc/rfc7800.html#section-3).
    pub fn require_jwk_key_binding(mut self, verifying_key: &VerifyingKey) -> Result<Self> {
        let jwk = jwk_from_p256(verifying_key)?;
        self.key_bind = Some(RequiredKeyBinding::Jwk(jwk));
        Ok(self)
    }

    /// Creates an SD-JWT with the provided data.
    pub async fn finish(self, alg: Algorithm, signing_key: &impl EcdsaKeySend) -> Result<SdJwt> {
        let SdJwtBuilder {
            mut encoder,
            disclosures,
            key_bind,
            mut header,
        } = self;
        encoder.add_sd_alg_property();
        let mut object = encoder.object;
        // Add key binding requirement as `cnf`.
        if let Some(key_bind) = key_bind {
            let key_bind = serde_json::to_value(key_bind)?;
            object
                .as_object_mut()
                .expect("encoder::object is a JSON Object")
                .insert("cnf".to_string(), key_bind);
        }

        header.alg = alg;

        let claims = serde_json::from_value::<SdJwtClaims>(object)
            .map_err(|e| Error::Deserialization(format!("invalid SD-JWT claims: {e}")))?;

        let verified_jwt = VerifiedJwt::sign(claims, header, signing_key).await?;

        Ok(SdJwt::new(verified_jwt, disclosures))
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    mod marking_properties_as_concealable {
        use super::*;

        mod that_exist {
            use super::*;

            mod on_top_level {
                use super::*;

                #[test]
                fn can_be_done_for_object_values() {
                    let result = SdJwtBuilder::new(json!({ "address": {} }))
                        .unwrap()
                        .make_concealable("/address");

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                        .unwrap()
                        .make_concealable("/nationalities");

                    assert!(result.is_ok());
                }
            }

            mod as_subproperties {
                use super::*;

                #[test]
                fn can_be_done_for_object_values() {
                    let result = SdJwtBuilder::new(json!({ "address": { "country": "US" } }))
                        .unwrap()
                        .make_concealable("/address/country");

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = SdJwtBuilder::new(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
                    .unwrap()
                    .make_concealable("/address/contact_person/0");

                    assert!(result.is_ok());
                }
            }
        }

        mod that_do_not_exist {
            use super::*;

            mod on_top_level {
                use assert_matches::assert_matches;

                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = SdJwtBuilder::new(json!({})).unwrap().make_concealable("/email");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/email");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({}))
                        .unwrap()
                        .make_concealable("/nationalities/0");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/nationalities/0");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = SdJwtBuilder::new(json!({
                      "nationalities": ["US", "DE"]
                    }))
                    .unwrap()
                    .make_concealable("/nationalities/2");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/nationalities/2");
                }
            }

            mod as_subproperties {
                use assert_matches::assert_matches;

                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = SdJwtBuilder::new(json!({
                      "address": {}
                    }))
                    .unwrap()
                    .make_concealable("/address/region");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/address/region");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({
                      "address": {}
                    }))
                    .unwrap()
                    .make_concealable("/address/contact_person/2");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/address/contact_person/2");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = SdJwtBuilder::new(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
                    .unwrap()
                    .make_concealable("/address/contact_person/2");

                    assert_matches!(result, Err(Error::InvalidPath(path)) if path == "/address/contact_person/2");
                }
            }
        }
    }

    mod adding_decoys {
        use super::*;

        mod on_top_level {
            use super::*;

            #[test]
            fn can_add_zero_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({})).unwrap().add_decoys("", 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({})).unwrap().add_decoys("", 2);

                assert!(result.is_ok());
            }
        }

        mod for_subproperties {
            use super::*;

            #[test]
            fn can_add_zero_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "address": {} }))
                    .unwrap()
                    .add_decoys("/address", 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "address": {} }))
                    .unwrap()
                    .add_decoys("/address", 2);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_zero_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                    .unwrap()
                    .add_decoys("/nationalities", 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                    .unwrap()
                    .add_decoys("/nationalities", 2);

                assert!(result.is_ok());
            }
        }
    }
}
