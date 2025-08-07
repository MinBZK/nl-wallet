// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::VerifyingKey;
use serde::Serialize;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use crypto::EcdsaKeySend;
use crypto::x509::BorrowingCertificate;
use jwt::VerifiedJwt;
use jwt::jwk::jwk_from_p256;
use utils::vec_at_least::VecNonEmpty;

use crate::disclosure::Disclosure;
use crate::encoder::DEFAULT_SALT_SIZE;
use crate::encoder::SdObjectEncoder;
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
    disclosures: HashMap<String, Disclosure>,
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
            disclosures: HashMap::new(),
            header: Header {
                typ: Some(String::from(SD_JWT_HEADER_TYP)),
                ..Default::default()
            },
        })
    }

    /// Substitutes a value with the digest of its disclosure.
    ///
    /// ## Notes
    /// - `path`  indicates the claim paths pointing to the value that will be concealed.
    ///
    /// ## Example
    ///  ```rust
    ///  use attestation_types::claim_path::ClaimPath;
    ///  use serde_json::json;
    ///  use sd_jwt::builder::SdJwtBuilder;
    ///  use utils::vec_at_least::VecNonEmpty;
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
    ///   //conceals "id": "did:value"
    ///   .make_concealable(VecNonEmpty::try_from(vec![ClaimPath::SelectByKey(String::from("id"))]).unwrap()).unwrap()
    ///   //"abc": true
    ///   .make_concealable(VecNonEmpty::try_from(
    ///       vec![
    ///          ClaimPath::SelectByKey(String::from("claim1")),
    ///          ClaimPath::SelectByKey(String::from("abc"))
    ///       ]
    ///   ).unwrap()).unwrap()
    ///   //conceals "val_1"
    ///   .make_concealable(VecNonEmpty::try_from(
    ///       vec![
    ///          ClaimPath::SelectByKey(String::from("claim2")),
    ///          ClaimPath::SelectByIndex(0)
    ///       ]
    ///   ).unwrap()).unwrap();
    /// ```
    pub fn make_concealable(mut self, path: VecNonEmpty<ClaimPath>) -> Result<Self> {
        let disclosure = self.encoder.conceal(path)?;
        self.disclosures
            .insert(self.encoder.hasher.encoded_digest(disclosure.as_str()), disclosure);

        Ok(self)
    }

    /// Adds a decoy digest to the specified path.
    ///
    /// `path`  indicates the claim paths pointing to the value that will be concealed.
    ///
    /// Use `path` = &[] to add decoys to the top level.
    pub fn add_decoys(mut self, path: &[ClaimPath], number_of_decoys: usize) -> Result<Self> {
        self.encoder.add_decoys(path, number_of_decoys)?;

        Ok(self)
    }

    /// Creates an SD-JWT with the provided data.
    pub async fn finish(
        self,
        alg: Algorithm,
        vct_integrity: Integrity,
        issuer_signing_key: &impl EcdsaKeySend,
        issuer_certificates: Vec<BorrowingCertificate>,
        holder_pubkey: &VerifyingKey,
    ) -> Result<SdJwt> {
        let SdJwtBuilder {
            mut encoder,
            disclosures,
            mut header,
        } = self;
        encoder.add_sd_alg_property();

        header.alg = alg;
        header.x5c = Some(
            issuer_certificates
                .iter()
                .map(|cert| BASE64_STANDARD.encode(cert.to_vec()))
                .collect_vec(),
        );

        let mut claims = serde_json::from_value::<SdJwtClaims>(encoder.encode())?;
        claims.cnf = Some(RequiredKeyBinding::Jwk(jwk_from_p256(holder_pubkey)?));
        claims.vct_integrity = Some(vct_integrity);

        let verified_jwt = VerifiedJwt::sign(claims, header, issuer_signing_key).await?;

        Ok(SdJwt::new(verified_jwt, issuer_certificates, disclosures))
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use serde_json::json;

    use crate::error::Error;

    use super::*;

    mod marking_properties_as_concealable {
        use super::*;

        mod that_exist {
            use super::*;

            mod on_top_level {
                use super::*;

                #[test]
                fn can_be_done_for_object_values() {
                    let result = SdJwtBuilder::new(json!({ "address": {} })).unwrap().make_concealable(
                        vec![ClaimPath::SelectByKey(String::from("address"))]
                            .try_into()
                            .unwrap(),
                    );

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                        .unwrap()
                        .make_concealable(
                            vec![ClaimPath::SelectByKey(String::from("nationalities"))]
                                .try_into()
                                .unwrap(),
                        );

                    assert!(result.is_ok());
                }
            }

            mod as_subproperties {
                use super::*;

                #[test]
                fn can_be_done_for_object_values() {
                    let result = SdJwtBuilder::new(json!({ "address": { "country": "US" } }))
                        .unwrap()
                        .make_concealable(
                            vec![
                                ClaimPath::SelectByKey(String::from("address")),
                                ClaimPath::SelectByKey(String::from("country")),
                            ]
                            .try_into()
                            .unwrap(),
                        );

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = SdJwtBuilder::new(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("contact_person")),
                            ClaimPath::SelectByIndex(0),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert!(result.is_ok());
                }
            }
        }

        mod that_do_not_exist {
            use super::*;

            mod on_top_level {
                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = SdJwtBuilder::new(json!({}))
                        .unwrap()
                        .make_concealable(vec![ClaimPath::SelectByKey(String::from("email"))].try_into().unwrap());

                    assert_matches!(result, Err(Error::DisclosureNotFound(key, _)) if key == "email");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({})).unwrap().make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("nationalities")),
                            ClaimPath::SelectByIndex(0),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(Error::ParentNotFound(_)));
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = SdJwtBuilder::new(json!({
                      "nationalities": ["US", "DE"]
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("nationalities")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(Error::IndexOutOfBounds(2, _)));
                }
            }

            mod as_subproperties {
                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = SdJwtBuilder::new(json!({
                      "address": {}
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("region")),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(Error::DisclosureNotFound(key, _)) if key == "region");
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({
                      "address": {}
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("contact_person")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(Error::ParentNotFound(_)));
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = SdJwtBuilder::new(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("contact_person")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(Error::IndexOutOfBounds(2, _)));
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
                let result = SdJwtBuilder::new(json!({})).unwrap().add_decoys(&[], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({})).unwrap().add_decoys(&[], 2);

                assert!(result.is_ok());
            }
        }

        mod for_subproperties {
            use super::*;

            #[test]
            fn can_add_zero_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "address": {} }))
                    .unwrap()
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "address": {} }))
                    .unwrap()
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 2);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_zero_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                    .unwrap()
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({ "nationalities": ["US", "DE"] }))
                    .unwrap()
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 2);

                assert!(result.is_ok());
            }
        }
    }
}
