use std::collections::HashMap;

use error_category::ErrorCategory;
use sd_jwt::error::DecoderError;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use serde_json::Value;

/// All decoded claims of an SD-JWT or SD-JWT presentation, i.e. with disclosures substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdJwtClaims(HashMap<String, Value>);

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum SdJwtClaimsError {
    #[error("could not decode SD-JWT claims: {0}")]
    #[category(pd)]
    Decoding(#[source] DecoderError),

    #[error("could not convert SD-JWT claims to JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[source] serde_json::Error),
}

impl SdJwtClaims {
    pub fn from_verified_sd_jwt(verified_sd_jwt: &VerifiedSdJwt) -> Result<Self, SdJwtClaimsError> {
        let decoded_claims = verified_sd_jwt.decoded_claims().map_err(SdJwtClaimsError::Decoding)?;

        let claims = decoded_claims
            .claims
            .into_iter()
            .map(|(name, value)| Ok((name.into_inner(), serde_json::to_value(value)?)))
            .collect::<Result<_, _>>()
            .map_err(SdJwtClaimsError::JsonConversion)?;

        Ok(Self(claims))
    }

    pub fn from_unsigned_sd_jwt_presentation(
        sd_jwt_presentation: &UnsignedSdJwtPresentation,
    ) -> Result<Self, SdJwtClaimsError> {
        Self::from_verified_sd_jwt(sd_jwt_presentation.as_ref())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use sd_jwt::builder::SdJwtBuilder;
    use sd_jwt::sd_jwt::SdJwtVcClaims;
    use sd_jwt::sd_jwt::VerifiedSdJwt;
    use serde_json::Value;
    use serde_json::json;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::SdJwtClaims;

    fn example_claims_json() -> Value {
        json!({
            "iss": "https://issuer.example.com",
            "given_name": "Willeke",
            "birth_date": "1940-01-01",
            "nationalities": ["NL", "DE"],
            "address": {
                "street": "Turfmarkt",
                "locality": "Den Haag",
            },
        })
    }

    fn claims_from_json(json: Value) -> HashMap<String, Value> {
        let Value::Object(claims) = json else {
            panic!("claims should be a JSON object");
        };

        claims.into_iter().collect()
    }

    fn path(elements: &[ClaimPath]) -> VecNonEmpty<ClaimPath> {
        elements.to_vec().try_into().unwrap()
    }

    fn key(key: &str) -> ClaimPath {
        ClaimPath::SelectByKey(key.to_string())
    }

    /// Paths of the claims in [`example_claims_json`] that are made selectively disclosable. Nested claims have to be
    /// concealed before their parent.
    fn concealable_paths() -> Vec<VecNonEmpty<ClaimPath>> {
        vec![
            path(&[key("birth_date")]),
            path(&[key("nationalities"), ClaimPath::SelectByIndex(1)]),
            path(&[key("address"), key("locality")]),
            path(&[key("address")]),
        ]
    }

    fn example_sd_jwt() -> VerifiedSdJwt {
        let holder_key = SigningKey::generate();
        let issuer_keypair = Ca::generate_issuer_mock_ca().unwrap().generate_issuer_mock().unwrap();

        let claims = SdJwtVcClaims::example_from_json(
            holder_key.verifying_key(),
            example_claims_json(),
            &MockTimeGenerator::default(),
        );

        // Decoys within `address` have to be added before concealing `address` itself.
        let builder = SdJwtBuilder::new(claims).add_decoys(&[key("address")], 1).unwrap();

        concealable_paths()
            .into_iter()
            .try_fold(builder, SdJwtBuilder::make_concealable)
            .unwrap()
            .add_decoys(&[], 2)
            .unwrap()
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into_verified()
    }

    #[test]
    fn test_sd_jwt_claims_from_verified_sd_jwt() {
        let sd_jwt = example_sd_jwt();
        assert_eq!(sd_jwt.disclosures().len(), concealable_paths().len());

        let SdJwtClaims(claims) =
            SdJwtClaims::from_verified_sd_jwt(&sd_jwt).expect("SD-JWT claims should decode successfully");

        // All claims are decoded, without any `_sd` or `...` digests, and registered claims that are not part of the
        // flattened claims (such as `vct`, `iat` and `cnf`) are absent. Note that `iss` is not such a claim.
        assert_eq!(claims, claims_from_json(example_claims_json()));
    }

    #[test]
    fn test_sd_jwt_claims_from_unsigned_sd_jwt_presentation() {
        let presentation = example_sd_jwt()
            .into_presentation_builder()
            .disclose(&vec_nonempty![key("address"), key("street")])
            .unwrap()
            .finish();

        let SdJwtClaims(claims) = SdJwtClaims::from_unsigned_sd_jwt_presentation(&presentation)
            .expect("SD-JWT claims should decode successfully");

        // Only the claims that are not selectively disclosable and the disclosed claims are present.
        let expected = claims_from_json(json!({
            "iss": "https://issuer.example.com",
            "given_name": "Willeke",
            "nationalities": ["NL"],
            "address": {
                "street": "Turfmarkt",
            },
        }));
        assert_eq!(claims, expected);
    }
}
