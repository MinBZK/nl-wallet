//! Selective-Disclosure JWT (SD-JWT) utilities for issuance, verification, and presentation.
//!
//! # What this crate provides
//! - Types that model the SD-JWT (VC) specification with strong typing.
//! - A typed payload for SD-JWT VCs: `SdJwtVcClaims`.
//! - Helpers to conceal claims, add decoys, create disclosures, and decode them.
//! - Builders to issue SD-JWTs and to create SD-JWT presentations with a KB-JWT.
//!
//! # Key concepts and relationships
//! - Claims and hashing
//!   - [`SdJwtVcClaims`](sd_jwt::SdJwtVcClaims) is the canonical SD-JWT payload type as defined in <https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html>.
//!     It contains:
//!     - `_sd_alg`: the hash algorithm identifier (defaults to sha-256 if absent)
//!     - `cnf`: the holder binding
//!     - `vct`: the attestation type
//!     - metadata (`iss`, `iat`, `exp`/`nbf`, etc.)
//!     - `claims`: the selectively disclosable claim tree represented by `ClaimValue` and `ObjectClaims`.
//!   - The hash algorithm is selected via `SdAlg`; implementations live behind the [`hasher::Hasher`] trait. Currently,
//!     only `Sha256` is implemented via `Sha256Hasher`.
//! - JWT. The underlying JWT handling (sign/verify) is provided by the `jwt` crate. This crate uses:
//!   - `HeaderWithX5c` when issuing SD-JWTs (issuer certificate chain embedded in `x5c`),
//!   - the holder key binding lives in the `cnf` claim as a JWK (`RequiredKeyBinding`).
//! - SD-JWTs. Note: to be able to parse the examples from the spec, generics `<C, H>` are provided, but limited to
//!   what's needed in the examples/tests:
//!   - [`UnverifiedSdJwt`](sd_jwt::UnverifiedSdJwt): raw serialization received over an untrusted channel. Can be
//!     parsed and verified into a `VerifiedSdJwt` with verified disclosures.
//!   - [`SignedSdJwt`](builder::SignedSdJwt): freshly issued SD-JWT. Can be converted into `UnverifiedSdJwt` for
//!     transmission.
//!   - [`VerifiedSdJwt`](sd_jwt::VerifiedSdJwt): verified issuer-signed JWT with verified disclosures, indexed by
//!     digest.
//! - SD-JWT presentation and KB-JWT. A presentation combines a selected subset of disclosures with a KB-JWT binding the
//!   SD-JWT and session to the holder key.
//!   - [`UnsignedSdJwtPresentation`](sd_jwt::UnsignedSdJwtPresentation): is created by from a `VerifiedSdJwt` by
//!     `SdJwtPresentationBuilder` after selecting which disclosures to include. Turned into `SignedSdJwtPresentation`
//!     by signing a KB-JWT.
//!   - [`UnverifiedSdJwtPresentation`](sd_jwt::UnverifiedSdJwtPresentation): raw serialization received over an
//!     untrusted channel. Can be parsed and verified into a `VerifiedSdJwtPresentation` with verified disclosures and
//!     KB-JWT. Note that first the issuer-signed JWT must be parsed before the `KB-JWT` can be verified using the
//!     holder key from the `cnf` claim. If both signatures are verified, the disclosures will be parsed.
//!   - [`VerifiedSdJwtPresentation`](sd_jwt::VerifiedSdJwtPresentation): verified SD-JWT presentation with verified
//!     disclosures and verified KB-JWT.
//!
//! # Usage
//! - For issuance, issuer side
//!   1. Fill the `SdJwtVcClaims` (set `cnf` to contain the holder's public key).
//!   2. Create an `SdJwtBuilder` with the claims; call `make_concealable` for paths to hide; optionally `add_decoys`.
//!   3. Call `finish` with the issuer key to sign the SD-JWT into a `SignedSdJwt`.
//!   4. Issue the `SignedSdJwt` to the holder.
//! - For issuance, holder side
//!   1. Deserialize with `UnverifiedSdJwt::parse`.
//!   2. Verify the issuer JWT and certificate chain with `UnverifiedSdJwt::into_verified_against_trust_anchors`.
//!   3. Obtain decoded claims with `VerifiedSdJwt::decoded_claims()`.
//! - For presentation (disclosure), holder side
//!   1. Create an `SdJwtPresentationBuilder` using `VerifiedSdJwt::into_presentation_builder`.
//!   2. Call `disclose` for each path to be disclosed.
//!   3. Call then `finish` to obtain `UnsignedSdJwtPresentation`.
//!   4. Sign the SD-JWT presentation with the holder key (WSCD), producing a `SignedSdJwtPresentation`.
//!   5. Send the `SignedSdJwtPresentation` to the verifier.
//! - For verification, verifier side
//!   1. Deserialize with `UnverifiedSdJwtPresentation::parse`.
//!   2. Verify the SD-JWT and the KB-JWT with `UnverifiedSdJwtPresentation::into_verified_against_trust_anchors`.
//!   3. Obtain decoded claims with `VerifiedSdJwtPresentation::sd_jwt().decoded_claims()`.
//!
//! # Example; issuance, presentation, and verification
//! ```
//! # use attestation_types::claim_path::ClaimPath;
//! # use chrono::Utc;
//! # use crypto::server_keys::generate::Ca;
//! # use jwt::headers::HeaderWithX5c;
//! # use jwt::jwk::jwk_from_p256;
//! # use p256::ecdsa::{SigningKey, VerifyingKey};
//! # use rand::rngs::OsRng;
//! # use rustls_pki_types::TrustAnchor;
//! # use sd_jwt::builder::SdJwtBuilder;
//! # use sd_jwt::claims::ClaimName;
//! # use sd_jwt::claims::ClaimValue;
//! # use sd_jwt::disclosure::Disclosure;
//! # use sd_jwt::key_binding_jwt::KbVerificationOptions;
//! # use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
//! # use sd_jwt::key_binding_jwt::RequiredKeyBinding;
//! # use sd_jwt::sd_jwt::{SdJwtVcClaims, VerifiedSdJwt};
//! # use std::time::Duration;
//! # use utils::date_time_seconds::DateTimeSeconds;
//! # use utils::generator::TimeGenerator;
//! # use utils::vec_at_least::VecNonEmpty;
//! # use utils::vec_nonempty;
//!
//! # tokio_test::block_on(async {
//! // 1) Issuer constructs SD-JWT VC claims, including the holder's public key.
//! let holder_privkey = SigningKey::random(&mut OsRng);
//! let claims = SdJwtVcClaims {
//!     _sd_alg: None,
//!     cnf: RequiredKeyBinding::Jwk(jwk_from_p256(&holder_privkey.verifying_key())?),
//!     vct: "com:example:vct".into(),
//!     vct_integrity: None,
//!     iss: "https://issuer.example.com".parse()?,
//!     iat: DateTimeSeconds::from(Utc::now()),
//!     exp: None,
//!     nbf: None,
//!     attestation_qualification: None,
//!     status: None,
//!     claims: serde_json::from_value(serde_json::json!({
//!         "name": "alice"
//!     }))?,
//! };
//!
//! // Issuer setup
//! let ca = Ca::generate_issuer_mock_ca()?;
//! let issuer_keypair = ca.generate_issuer_mock()?;
//!
//! // 2) Issuer marks fields as concealable and signs with issuer key.
//! let signed = SdJwtBuilder::new(claims)
//!     .make_concealable(VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("name".into())])?)?
//!     // optionally add decoys with `add_decoys`
//!     .finish(&issuer_keypair)
//!     .await
//!     ?;
//!
//! // 3) Issuer sends/Holder receives the SD-JWT as `UnverifiedSdJwt`.
//! let unverified = signed.into_unverified();
//!
//! // 4) Holder parses and verifies SD-JWT against trust anchors.
//! let trust_anchors = vec![ca.to_trust_anchor()];
//! let verified = unverified.into_verified_against_trust_anchors(
//!     &trust_anchors,
//!     &TimeGenerator::default()
//! )?;
//!
//! // 4) Holder creates a presentation with (a subset of) disclosures and signs a KB-JWT.
//! let presentation = verified
//!     .into_presentation_builder()
//!     .disclose(&vec_nonempty![ClaimPath::SelectByKey("name".into())])?
//!     .finish();
//! let kb = KeyBindingJwtBuilder::new("https://verifier.example.com".into(), "nonce-123".into());
//! let signed_presentation = presentation.sign(kb, &holder_privkey, &TimeGenerator::default()).await?;
//!
//! // 5) Verifier verifies the presentation (SD-JWT via trust anchors + KB-JWT via cnf JWK) and decodes claims.
//! let verified_presentation = signed_presentation.into_unverified().into_verified_against_trust_anchors(
//!     &trust_anchors,
//!     &KbVerificationOptions {
//!         expected_aud: "https://verifier.example.com",
//!         expected_nonce: "nonce-123",
//!         iat_leeway: Duration::ZERO,
//!         iat_acceptance_window: Duration::from_secs(300),
//!     },
//!     &TimeGenerator::default()
//! )?;
//! let disclosed = verified_presentation.sd_jwt().decoded_claims()?;
//! assert_eq!(disclosed.claims.get(&ClaimName::try_from("name").unwrap()), Some(&ClaimValue::String("alice".to_owned())));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
pub mod builder;
pub mod claims;
mod decoder;
pub mod disclosure;
mod encoder;
pub mod error;
pub mod hasher;
pub mod key_binding_jwt;
mod sd_alg;
pub mod sd_jwt;

#[cfg(any(test, feature = "examples"))]
pub mod examples;

#[cfg(any(test, feature = "test"))]
pub mod test;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use rstest::rstest;
    use serde_json::json;

    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::generate::Ca;
    use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use crate::error::ClaimError;
    use crate::sd_jwt::SdJwtVcClaims;
    use crate::test::conceal_and_sign;
    use crate::test::disclose_claims;

    fn test_object() -> SdJwtVcClaims {
        let input_object = json!({
            "vct": "com:example:1",
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "cnf": {
                "jwk": {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                    "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            },
            "root_value": 1,
            "root_array": [
                {
                    "array_object_value": 1,
                },
                {
                    "array_object_value": 2,
                },
                {
                    "array_object_value": 3,
                }
            ],
            "root_object": {
                "object_value": 4,
                "object_array": [
                    {
                        "nested_object_value": 4,
                    },
                    {
                        "nested_object_value": 5,
                    },
                    {
                        "nested_object_value": 6,
                    }
                ]
            }
        });

        serde_json::from_value(input_object).unwrap()
    }

    #[test]
    fn test_encode_decode() {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        // conceal all claims, and encode as an SD-JWT
        let signed_sd_jwt = conceal_and_sign(&issuer_keypair, test_object(), all_claims());

        // disclose all claims
        let unsigned_sd_jwt = disclose_claims(signed_sd_jwt.into_verified(), &all_claims());

        // decode the disclosed SD-JWT
        let claims = unsigned_sd_jwt.as_ref().decoded_claims().unwrap();

        assert_eq!(&claims, test_object().claims());
    }

    #[rstest]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Allowed, true)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Never, false)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_array".to_string())], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_array".to_string()), ClaimPath::SelectAll], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_object".to_string())], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(all_claims(), vec![ClaimPath::SelectByKey("root_object".to_string()), ClaimPath::SelectByKey("object_array".to_string()), ClaimPath::SelectAll], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Always, false)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Allowed, true)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_value".to_string())], ClaimSelectiveDisclosureMetadata::Never, true)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_array".to_string())], ClaimSelectiveDisclosureMetadata::Never, true)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_array".to_string()), ClaimPath::SelectAll], ClaimSelectiveDisclosureMetadata::Always, true)] // ClaimError::ClaimSelectiveDisclosureMetadataMismatch
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_object".to_string())], ClaimSelectiveDisclosureMetadata::Always, true)]
    #[case(selected_claims(), vec![ClaimPath::SelectByKey("root_object".to_string()), ClaimPath::SelectByKey("object_array".to_string()), ClaimPath::SelectAll], ClaimSelectiveDisclosureMetadata::Always, false)]
    #[case(double_concealed_array_claims(), vec![ClaimPath::SelectByKey("root_array".to_string()), ClaimPath::SelectAll, ClaimPath::SelectByKey("array_object_value".to_string())], ClaimSelectiveDisclosureMetadata::Always, false)]
    fn test_verify_selective_disclosability(
        #[case] concealed_claims: Vec<VecNonEmpty<ClaimPath>>,
        #[case] path: Vec<ClaimPath>,
        #[case] sd: ClaimSelectiveDisclosureMetadata,
        #[case] should_succeed: bool,
    ) {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        let input = test_object();

        // conceal claims, and encode as an SD-JWT
        let sd_jwt = conceal_and_sign(&issuer_keypair, input, concealed_claims);
        let verified_sd_jwt = sd_jwt.into_verified();

        let metadata = HashMap::from_iter([(path.clone(), sd)]);

        let result = verified_sd_jwt.verify_selective_disclosability(&path, &metadata);
        if should_succeed {
            result.unwrap();
        } else {
            result.unwrap_err();
        }
    }

    #[rstest]
    #[case(all_claims(),
           vec![],
           |actual| assert_eq!(actual, ClaimError::EmptyPath)
    )]
    #[case(all_claims(),
           vec![ClaimPath::SelectAll],
           |actual| assert_matches!(actual, ClaimError::UnexpectedElement(_, path) if path == vec![ClaimPath::SelectAll])
    )]
    #[case(all_claims(),
           vec![ClaimPath::SelectByIndex(0)],
           |actual| assert_eq!(actual, ClaimError::UnsupportedTraversalPath(ClaimPath::SelectByIndex(0)))
    )]
    #[case(all_claims(),
           vec![ClaimPath::SelectByKey("missing_root_value".to_string())],
           |actual| assert_matches!(actual, ClaimError::ObjectFieldNotFound(claim_name, _) if claim_name.as_str() == "missing_root_value")
    )]
    #[case(all_claims(),
           vec![ClaimPath::SelectByKey("root_object".to_string()), ClaimPath::SelectByKey("missing_nested_value".to_string())],
           |actual| assert_matches!(actual, ClaimError::ObjectFieldNotFound(claim_name, _) if claim_name.as_ref() == "missing_nested_value")
    )]
    #[case(all_claims(),
           vec![ClaimPath::SelectByKey("root_object".to_string()), ClaimPath::SelectByKey("missing_nested_value".to_string()), ClaimPath::SelectAll],
           |actual| assert_matches!(actual, ClaimError::ObjectFieldNotFound(claim_name, _) if claim_name.as_ref() == "missing_nested_value")
    )]
    fn test_verify_selective_disclosability_errors(
        #[case] concealed_claims: Vec<VecNonEmpty<ClaimPath>>,
        #[case] path: Vec<ClaimPath>,
        #[case] verify_expected_error: impl FnOnce(ClaimError),
    ) {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        let input = test_object();

        // conceal claims, and encode as an SD-JWT
        let sd_jwt = conceal_and_sign(&issuer_keypair, input, concealed_claims);
        let verified_sd_jwt = sd_jwt.into_verified();

        let error = verified_sd_jwt
            .verify_selective_disclosability(&path, &HashMap::new())
            .unwrap_err();

        verify_expected_error(error);
    }

    fn all_claims() -> Vec<VecNonEmpty<ClaimPath>> {
        vec![
            vec_nonempty![ClaimPath::SelectByKey("root_value".to_string())],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(0)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(1)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(2)
            ],
            vec_nonempty![ClaimPath::SelectByKey("root_array".to_string())],
            vec_nonempty![
                ClaimPath::SelectByKey("root_object".to_string()),
                ClaimPath::SelectByKey("object_value".to_string())
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_object".to_string()),
                ClaimPath::SelectByKey("object_array".to_string()),
                ClaimPath::SelectByIndex(0)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_object".to_string()),
                ClaimPath::SelectByKey("object_array".to_string()),
                ClaimPath::SelectByIndex(1)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_object".to_string()),
                ClaimPath::SelectByKey("object_array".to_string()),
                ClaimPath::SelectByIndex(2)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_object".to_string()),
                ClaimPath::SelectByKey("object_array".to_string())
            ],
            vec_nonempty![ClaimPath::SelectByKey("root_object".to_string())],
        ]
    }

    fn selected_claims() -> Vec<VecNonEmpty<ClaimPath>> {
        vec![
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(0)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(1)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(2)
            ],
            vec_nonempty![ClaimPath::SelectByKey("root_object".to_string())],
        ]
    }

    fn double_concealed_array_claims() -> Vec<VecNonEmpty<ClaimPath>> {
        vec![
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(0)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(0)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(1)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(1)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(2)
            ],
            vec_nonempty![
                ClaimPath::SelectByKey("root_array".to_string()),
                ClaimPath::SelectByIndex(2)
            ],
        ]
    }
}
