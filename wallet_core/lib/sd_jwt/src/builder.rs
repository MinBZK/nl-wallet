use std::fmt::Display;

use indexmap::IndexMap;
use serde_with::SerializeDisplay;

use attestation_types::claim_path::ClaimPath;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use jwt::JwtTyp;
use jwt::SignedJwt;
use jwt::headers::HeaderWithX5c;
use utils::vec_at_least::VecNonEmpty;

use crate::disclosure::Disclosure;
use crate::encoder::DEFAULT_SALT_SIZE;
use crate::encoder::SdObjectEncoder;
use crate::error::EncoderError;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::sd_jwt::SdJwtVcClaims;
use crate::sd_jwt::UnverifiedSdJwt;
use crate::sd_jwt::VerifiedSdJwt;

// <https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-10.html#name-jose-header>
const SD_JWT_HEADER_TYP: &str = "dc+sd-jwt";

impl JwtTyp for SdJwtVcClaims {
    const TYP: &'static str = SD_JWT_HEADER_TYP;
}

/// A freshly issued SD-JWT consisting of an issuer-signed JWT (with `x5c`) and its disclosures.
///
/// Formats as `<Issuer-signed JWT>~<Disclosure>~...~<Disclosure>~`.
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
    /// Converts signed SD-JWT into its unverified form.
    ///
    /// This is used to be able to serialize and deserialize a wrapper type with the same content.
    pub fn into_unverified(self) -> UnverifiedSdJwt {
        self.into()
    }

    /// Converts signed SD-JWT into a verified SD-JWT without an additional verification step.
    ///
    /// This is safe because the value was just signed.
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

        // the `SignedSdJwt` was just created by our own builder, so the hasher should always be implemented
        let hasher = issuer_signed.payload()._sd_alg.unwrap_or_default().hasher().unwrap();
        let disclosures = value
            .disclosures
            .into_iter()
            .map(|d| (hasher.encoded_digest(&d.encoded), d))
            .collect::<IndexMap<_, _>>();
        VerifiedSdJwt::dangerous_new(issuer_signed, disclosures)
    }
}

/// Builder to create an issuable SD-JWT:
/// - mark claims as concealable with [`SdJwtBuilder::make_concealable`],
/// - optionally add decoys with [`SdJwtBuilder::add_decoys`],
/// - call [`SdJwtBuilder::finish`] to sign with the issuer certificate (x5c) and produce a [`SignedSdJwt`].
///
/// # Example:
/// ```
/// # use attestation_types::claim_path::ClaimPath;
/// # use chrono::Utc;
/// # use crypto::server_keys::generate::Ca;
/// # use jwt::confirmation::ConfirmationClaim;
/// # use p256::ecdsa::SigningKey;
/// # use rand_core::OsRng;
/// # use sd_jwt::builder::SdJwtBuilder;
/// # use sd_jwt::sd_jwt::SdJwtVcClaims;
/// # use utils::date_time_seconds::DateTimeSeconds;
///
///  # tokio_test::block_on(async {
/// let holder_key = SigningKey::random(&mut OsRng);
/// let claims = SdJwtVcClaims {
///     _sd_alg: None,
///     cnf: ConfirmationClaim::from_verifying_key(&holder_key.verifying_key())?,
///     vct: "urn:example:vct".into(),
///     vct_integrity: None,
///     iss: "https://issuer.example.com".parse()?,
///     iat: DateTimeSeconds::from(Utc::now()),
///     exp: None,
///     nbf: None,
///     attestation_qualification: None,
///     status: None,
///     claims: serde_json::from_value(serde_json::json!({
///         "name": "alice"
///     }))?,
/// };
/// let builder = SdJwtBuilder::new(claims)
///     .make_concealable(vec![ClaimPath::SelectByKey("name".into())].try_into()?)?;
///
/// let ca = Ca::generate_issuer_mock_ca()?;
/// let issuer_keypair = ca.generate_issuer_mock()?;
/// let signed = builder.finish(&issuer_keypair).await?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
#[derive(Debug)]
pub struct SdJwtBuilder<H> {
    encoder: SdObjectEncoder<H>,
    disclosures: Vec<Disclosure>,
}

impl SdJwtBuilder<Sha256Hasher> {
    /// Creates a new [`SdJwtBuilder`] with `sha-256` hash function.
    pub fn new(claims: SdJwtVcClaims) -> Self {
        Self::new_with_hasher(claims, Sha256Hasher)
    }
}

impl<H: Hasher> SdJwtBuilder<H> {
    /// Creates a new [`SdJwtBuilder`] with custom hash function to create digests.
    pub fn new_with_hasher(claims: SdJwtVcClaims, hasher: H) -> Self {
        Self::new_with_hasher_and_salt_size(claims, hasher, DEFAULT_SALT_SIZE)
    }

    /// Creates a new [`SdJwtBuilder`] with custom hash function to create digests, and custom salt size.
    pub fn new_with_hasher_and_salt_size(claims: SdJwtVcClaims, hasher: H, salt_size: usize) -> Self {
        let encoder = SdObjectEncoder::with_custom_hasher_and_salt_size(claims, hasher, salt_size);
        Self {
            encoder,
            disclosures: Vec::new(),
        }
    }

    /// Substitutes a value with the digest of its disclosure.
    ///
    /// # Example
    /// ```
    /// # use attestation_types::claim_path::ClaimPath;
    /// # use chrono::Utc;
    /// # use jwt::confirmation::ConfirmationClaim;
    /// # use p256::ecdsa::SigningKey;
    /// # use serde_json::json;
    /// # use sd_jwt::builder::SdJwtBuilder;
    /// # use sd_jwt::sd_jwt::SdJwtVcClaims;
    /// # use utils::date_time_seconds::DateTimeSeconds;
    /// # use utils::vec_at_least::VecNonEmpty;
    /// # use rand_core::OsRng;
    ///
    /// let builder = SdJwtBuilder::new(SdJwtVcClaims {
    ///     _sd_alg: None,
    ///     cnf: ConfirmationClaim::from_verifying_key(&SigningKey::random(&mut OsRng).verifying_key())?,
    ///     vct: "com:example:vct".into(),
    ///     vct_integrity: None,
    ///     iss: "https://issuer.example.com".parse()?  ,
    ///     iat: DateTimeSeconds::from(Utc::now()),
    ///     exp: None,
    ///     nbf: None,
    ///     attestation_qualification: None,
    ///     status: None,
    ///     claims: serde_json::from_value(serde_json::json!({
    ///         "name": "alice",
    ///         "address": {
    ///             "house_number": 1
    ///         },
    ///         "nationalities": ["Dutch", "Belgian"]
    ///     }))?,
    /// })
    /// // conceals "name": "alice"
    /// .make_concealable(VecNonEmpty::try_from(vec![ClaimPath::SelectByKey(String::from("name"))])?)?
    /// // "house_number": 1
    /// .make_concealable(VecNonEmpty::try_from(
    ///     vec![
    ///        ClaimPath::SelectByKey(String::from("address")),
    ///        ClaimPath::SelectByKey(String::from("house_number"))
    ///     ]
    /// )?)?
    /// // conceals "Dutch"
    /// .make_concealable(VecNonEmpty::try_from(
    ///     vec![
    ///        ClaimPath::SelectByKey(String::from("nationalities")),
    ///        ClaimPath::SelectByIndex(0)
    ///     ]
    /// )?)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn make_concealable(mut self, path: VecNonEmpty<ClaimPath>) -> Result<Self, EncoderError> {
        let disclosure = self.encoder.conceal(path)?;
        self.disclosures.push(disclosure);
        Ok(self)
    }

    /// Adds a decoy digest to the specified path.
    ///
    /// `path`  indicates the claim paths pointing to the value that will be concealed.
    ///
    /// Use `path` = &[] to add decoys to the top level.
    pub fn add_decoys(mut self, path: &[ClaimPath], number_of_decoys: usize) -> Result<Self, EncoderError> {
        self.encoder.add_decoys(path, number_of_decoys)?;
        Ok(self)
    }

    /// Creates an SD-JWT by encoding selected disclosures and signing the issuer-signed part with the provided
    /// certificate/keypair.
    ///
    /// The resulting [`SignedSdJwt`] embeds the issuer certificate chain in the `x5c` header and formats as
    /// `<Issuer-signed JWT>~<Disclosure>~...~<Disclosure>~`.
    pub async fn finish(self, issuer_keypair: &KeyPair<impl EcdsaKey>) -> Result<SignedSdJwt, EncoderError> {
        let claims = self.encoder.encode();
        let issuer_signed = SignedJwt::sign_with_certificate(&claims, issuer_keypair).await?;
        Ok(SignedSdJwt {
            issuer_signed,
            disclosures: self.disclosures,
        })
    }
}

#[cfg(feature = "examples")]
mod examples {
    use futures::FutureExt;
    use p256::ecdsa::VerifyingKey;

    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::KeyPair;
    use utils::generator::mock::MockTimeGenerator;

    use crate::sd_jwt::SdJwtVcClaims;

    use super::SdJwtBuilder;
    use super::SignedSdJwt;

    impl SignedSdJwt {
        pub fn pid_example(issuer_keypair: &KeyPair, holder_pubkey: &VerifyingKey) -> Self {
            let claims = SdJwtVcClaims::pid_example(holder_pubkey, &MockTimeGenerator::default());

            // issuer signs SD-JWT
            SdJwtBuilder::new(claims)
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
                .finish(issuer_keypair)
                .now_or_never()
                .unwrap()
                .unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;

    use utils::generator::mock::MockTimeGenerator;

    use super::*;

    fn builder_from_json(object: serde_json::Value) -> SdJwtBuilder<Sha256Hasher> {
        SdJwtBuilder::new(SdJwtVcClaims::example_from_json(
            SigningKey::random(&mut OsRng).verifying_key(),
            object,
            &MockTimeGenerator::default(),
        ))
    }

    mod marking_properties_as_concealable {
        use super::*;

        mod that_exist {
            use super::*;

            mod on_top_level {
                use super::*;

                #[test]
                fn can_be_done_for_object_values() {
                    let result = builder_from_json(json!({ "address": {} })).make_concealable(
                        vec![ClaimPath::SelectByKey(String::from("address"))]
                            .try_into()
                            .unwrap(),
                    );

                    assert!(result.is_ok());
                }

                #[test]
                fn can_be_done_for_array_elements() {
                    let result = builder_from_json(json!({ "nationalities": ["US", "DE"] })).make_concealable(
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
                    let result = builder_from_json(json!({ "address": { "country": "US" } })).make_concealable(
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
                    let result = builder_from_json(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
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
                use crate::error::ClaimError;

                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = builder_from_json(json!({}))
                        .make_concealable(vec![ClaimPath::SelectByKey(String::from("email"))].try_into().unwrap());

                    assert_matches!(result, Err(EncoderError::ClaimStructure(ClaimError::ObjectFieldNotFound(key, _))) if key == "email".parse().unwrap());
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = builder_from_json(json!({})).make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("nationalities")),
                            ClaimPath::SelectByIndex(0),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(EncoderError::ClaimStructure(ClaimError::ParentNotFound(_))));
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = builder_from_json(json!({
                      "nationalities": ["US", "DE"]
                    }))
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("nationalities")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(
                        result,
                        Err(EncoderError::ClaimStructure(ClaimError::IndexOutOfBounds(2, _)))
                    );
                }
            }

            mod as_subproperties {
                use crate::error::ClaimError;

                use super::*;

                #[test]
                fn returns_an_error_for_nonexistant_object_paths() {
                    let result = builder_from_json(json!({
                      "address": {}
                    }))
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("region")),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(EncoderError::ClaimStructure(ClaimError::ObjectFieldNotFound(key, _))) if key == "region".parse().unwrap());
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_paths() {
                    let result = builder_from_json(json!({
                      "address": {}
                    }))
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("contact_person")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(result, Err(EncoderError::ClaimStructure(ClaimError::ParentNotFound(_))));
                }

                #[test]
                fn returns_an_error_for_nonexistant_array_entries() {
                    let result = builder_from_json(json!({
                      "address": { "contact_person": [ "Jane Dow", "John Doe" ] }
                    }))
                    .make_concealable(
                        vec![
                            ClaimPath::SelectByKey(String::from("address")),
                            ClaimPath::SelectByKey(String::from("contact_person")),
                            ClaimPath::SelectByIndex(2),
                        ]
                        .try_into()
                        .unwrap(),
                    );

                    assert_matches!(
                        result,
                        Err(EncoderError::ClaimStructure(ClaimError::IndexOutOfBounds(2, _)))
                    );
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
                let result = builder_from_json(json!({})).add_decoys(&[], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = builder_from_json(json!({})).add_decoys(&[], 2);

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
                let result = builder_from_json(json!({ "address": {} }))
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_object_value_decoys_for_a_path() {
                let result = builder_from_json(json!({ "address": {} }))
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("address"))], 2);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_zero_array_element_decoys_for_a_path() {
                let result = builder_from_json(json!({ "nationalities": ["US", "DE"] }))
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 0);

                assert!(result.is_ok());
            }

            #[test]
            fn can_add_array_element_decoys_for_a_path() {
                let result = builder_from_json(json!({ "nationalities": ["US", "DE"] }))
                    .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 2);

                assert!(result.is_ok());
            }
        }
    }
}
