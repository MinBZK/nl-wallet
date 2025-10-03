use std::fmt::Display;

use indexmap::IndexMap;
use p256::ecdsa::VerifyingKey;
use serde::Serialize;
use serde_with::SerializeDisplay;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::headers::HeaderWithX5c;
use jwt::jwk::jwk_from_p256;
use utils::vec_at_least::VecNonEmpty;

use crate::disclosure::Disclosure;
use crate::encoder::DEFAULT_SALT_SIZE;
use crate::encoder::SdObjectEncoder;
use crate::error::Result;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::key_binding_jwt::RequiredKeyBinding;
use crate::sd_jwt::SdJwtVcClaims;
use crate::sd_jwt::UnverifiedSdJwt;
use crate::sd_jwt::VerifiedSdJwt;

const SD_JWT_HEADER_TYP: &str = "dc+sd-jwt";

impl JwtTyp for SdJwtVcClaims {
    const TYP: &'static str = SD_JWT_HEADER_TYP;
}

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay)]
pub struct SignedSdJwt {
    issuer_signed: SignedJwt<SdJwtVcClaims, HeaderWithX5c>,
    disclosures: Vec<Disclosure>,
}

impl Display for SignedSdJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &std::iter::once(self.issuer_signed.as_ref().serialization())
                .chain(self.disclosures.iter().map(|d| d.encoded()))
                .map(|s| format!("{s}~"))
                .collect::<String>(),
        )
    }
}

impl SignedSdJwt {
    pub fn into_unverified(self) -> UnverifiedSdJwt {
        self.into()
    }

    pub fn into_verified(self) -> VerifiedSdJwt {
        self.into()
    }
}

impl From<SignedSdJwt> for UnverifiedSdJwt {
    fn from(value: SignedSdJwt) -> Self {
        let issuer_signed = value.issuer_signed.into_unverified();
        let disclosures = value.disclosures.iter().map(ToString::to_string).collect();
        UnverifiedSdJwt::new(issuer_signed, disclosures)
    }
}

impl From<SignedSdJwt> for VerifiedSdJwt {
    fn from(value: SignedSdJwt) -> Self {
        let issuer_signed = value.issuer_signed.into_verified();
        // the SignedSdJwt was just created by our own builder, so the hasher should always be implemented
        let hasher = issuer_signed.payload()._sd_alg.unwrap_or_default().hasher().unwrap();
        let disclosures = value
            .disclosures
            .into_iter()
            .map(|d| (hasher.encoded_digest(&d.encoded), d))
            .collect::<IndexMap<_, _>>();
        VerifiedSdJwt::dangerous_new(issuer_signed, disclosures)
    }
}

/// Builder structure to create an issuable SD-JWT.
#[derive(Debug)]
pub struct SdJwtBuilder<H> {
    encoder: SdObjectEncoder<H>,
    disclosures: Vec<Disclosure>,
}

impl SdJwtBuilder<Sha256Hasher> {
    /// Creates a new [`SdJwtBuilder`] with `sha-256` hash function.
    ///
    /// ## Error
    /// Returns [`Error::DataTypeMismatch`] if `object` is not a valid JSON object.
    pub fn new<T: Serialize>(object: T) -> Result<Self> {
        Self::new_with_hasher(object, Sha256Hasher)
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
            disclosures: Vec::new(),
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
    ///   "iss": "https://issuer.example.com/",
    ///   "iat": 1683000000,
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
        self.disclosures.push(disclosure);
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
        vct_integrity: Integrity,
        issuer_keypair: &KeyPair<impl EcdsaKey>,
        holder_pubkey: &VerifyingKey,
    ) -> Result<SignedSdJwt> {
        let SdJwtBuilder {
            mut encoder,
            disclosures,
        } = self;
        encoder.add_sd_alg_property();

        let mut claims = encoder.encode();
        claims.cnf = Some(RequiredKeyBinding::Jwk(jwk_from_p256(holder_pubkey)?));
        claims.vct_integrity = Some(vct_integrity);

        let issuer_signed = SignedJwt::sign_with_certificate(&claims, issuer_keypair).await?;
        Ok(SignedSdJwt {
            issuer_signed,
            disclosures,
        })
    }
}

#[cfg(feature = "examples")]
mod examples {
    use futures::FutureExt;
    use p256::ecdsa::VerifyingKey;
    use serde_json::json;
    use ssri::Integrity;

    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::KeyPair;
    use crypto::utils::random_string;

    use super::SdJwtBuilder;
    use super::SignedSdJwt;

    impl SignedSdJwt {
        pub fn pid_example(issuer_keypair: &KeyPair, holder_public_key: &VerifyingKey) -> Self {
            let object = json!({
                "vct": "urn:eudi:pid:nl:1",
                "iat": 1683000000,
                "exp": 1883000000,
                "iss": "https://cert.issuer.example.com",
                "attestation_qualification": "QEAA",
                "bsn": "999999999",
                "recovery_code": "cff292503cba8c4fbf2e5820dcdc468ae00f40c87b1af35513375800128fc00d",
                "given_name": "Willeke Liselotte",
                "family_name": "De Bruijn",
                "birthdate": "1940-01-01"
            });

            // issuer signs SD-JWT
            SdJwtBuilder::new(object)
                .unwrap()
                .make_concealable(
                    vec![ClaimPath::SelectByKey(String::from("family_name"))]
                        .try_into()
                        .unwrap(),
                )
                .unwrap()
                .make_concealable(vec![ClaimPath::SelectByKey(String::from("bsn"))].try_into().unwrap())
                .unwrap()
                .add_decoys(&[], 2)
                .unwrap()
                .finish(Integrity::from(random_string(32)), issuer_keypair, holder_public_key)
                .now_or_never()
                .unwrap()
                .unwrap()
        }
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
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
                        "address": {}
                    }))
                    .unwrap()
                    .make_concealable(
                        vec![ClaimPath::SelectByKey(String::from("address"))]
                            .try_into()
                            .unwrap(),
                    );

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
                        "nationalities": ["US", "DE"]
                    }))
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
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
                        "address": { "country": "US" }
                    }))
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
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
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
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000
                    }))
                    .unwrap()
                    .make_concealable(vec![ClaimPath::SelectByKey(String::from("email"))].try_into().unwrap());

                    assert_matches!(result, Err(Error::ObjectFieldNotFound(key, _)) if key == "email".parse().unwrap());
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000
                    }))
                    .unwrap()
                    .make_concealable(
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
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
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
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
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

                    assert_matches!(result, Err(Error::ObjectFieldNotFound(key, _)) if key == "region".parse().unwrap());
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = SdJwtBuilder::new(json!({
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
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
                        "iss": "https://issuer.example.com/",
                        "iat": 1683000000,
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
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000
                }))
                .unwrap()
                .add_decoys(&[], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000
                }))
                .unwrap()
                .add_decoys(&[], 2);

                assert!(result.is_ok());
                assert_eq!(
                    result.unwrap().encoder.object_claims()._sd.as_ref().unwrap().len(),
                    2.try_into().unwrap()
                );
            }
        }

        mod for_subproperties {
            use super::*;

            #[test]
            fn can_add_zero_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000,
                    "address": {}
                }))
                .unwrap()
                .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000,
                    "address": {}
                }))
                .unwrap()
                .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 2);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_zero_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000,
                    "nationalities": ["US", "DE"]
                }))
                .unwrap()
                .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_array_element_decoys_for_a_path() {
                let result = SdJwtBuilder::new(json!({
                    "iss": "https://issuer.example.com/",
                    "iat": 1683000000,
                    "nationalities": ["US", "DE"]
                }))
                .unwrap()
                .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 2);

                assert!(result.is_ok());
            }
        }
    }
}
