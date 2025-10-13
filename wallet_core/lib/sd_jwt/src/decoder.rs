use indexmap::IndexMap;
use itertools::Itertools;

use utils::vec_at_least::VecAtLeastN;

use crate::claims::ArrayClaim;
use crate::claims::ClaimValue;
use crate::claims::ObjectClaims;
use crate::disclosure::Disclosure;
use crate::error::Error;
use crate::error::Result;
use crate::sd_jwt::SdJwtVcClaims;

/// Substitutes digests in an [`SdJwtClaims`] by their corresponding claim values provided by disclosures.
pub struct SdObjectDecoder;

impl SdObjectDecoder {
    /// Decodes [`SdJwtClaims`] by substituting the digests with their corresponding claim values provided by
    /// `disclosures`.
    pub fn decode(sd_jwt_claims: &SdJwtVcClaims, disclosures: &IndexMap<String, Disclosure>) -> Result<SdJwtVcClaims> {
        // Clone the disclosures locally so we can mutate the HashMap
        let mut disclosures = disclosures.clone();

        // Decode all claims from the SD-JWT
        let claims = sd_jwt_claims.claims().decode(&mut disclosures)?;

        // All disclosures should have been resolved
        if !disclosures.is_empty() {
            return Err(Error::UnreferencedDisclosures(disclosures.into_keys().collect()));
        }

        // Construct a new SdJwtClaims with the decoded claims and without "_sd_alg" claim
        let sd_jwt_claims = SdJwtVcClaims {
            claims: ClaimValue::Object(claims),
            _sd_alg: None,
            ..sd_jwt_claims.clone()
        };

        Ok(sd_jwt_claims)
    }
}

impl ObjectClaims {
    pub fn decode(&self, disclosures: &mut IndexMap<String, Disclosure>) -> Result<Self> {
        let mut disclosed_claims = self
            ._sd
            .iter()
            .flat_map(VecAtLeastN::iter)
            .filter_map(|digest| disclosures.shift_remove(digest).map(|disclosure| (digest, disclosure)))
            .map(|(digest, disclosure)| {
                // Verify that the matching disclosure discloses an object property
                let (_, claim_name, claim_value) = disclosure.content.try_as_object_property(digest)?;
                Ok((claim_name.clone(), claim_value.clone()))
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        // Decode the disclosed claims here
        for disclosed_claim in disclosed_claims.values_mut() {
            *disclosed_claim = disclosed_claim.decode(disclosures)?;
        }

        for (claim_name, claim_value) in &self.claims {
            disclosed_claims.insert(claim_name.clone(), claim_value.decode(disclosures)?);
        }

        let result = Self {
            claims: disclosed_claims,
            ..Default::default()
        };

        Ok(result)
    }
}

impl ClaimValue {
    pub fn decode(&self, disclosures: &mut IndexMap<String, Disclosure>) -> Result<Self> {
        match self {
            ClaimValue::Array(claims) => {
                let decoded_claims = claims
                    .iter()
                    .map(|claim| claim.decode(disclosures))
                    .flatten_ok()
                    .collect::<Result<Vec<_>>>()?;

                Ok(ClaimValue::Array(decoded_claims))
            }
            ClaimValue::Object(object) => Ok(ClaimValue::Object(object.decode(disclosures)?)),
            _ => Ok(self.clone()),
        }
    }
}

impl ArrayClaim {
    pub fn decode(&self, disclosures: &mut IndexMap<String, Disclosure>) -> Result<Option<Self>> {
        let decoded_claim = match self {
            ArrayClaim::Hash(digest) => match disclosures.shift_remove(digest) {
                Some(disclosure) => {
                    // Verify that the matching disclosure discloses an array element
                    let (_, array_claim) = disclosure.content.try_as_array_element(digest)?;
                    array_claim.decode(disclosures)?
                }
                None => None,
            },
            ArrayClaim::Value(claim_value) => Some(ArrayClaim::Value(claim_value.decode(disclosures)?)),
        };
        Ok(decoded_claim)
    }
}

// TODO: [PVW-4138] Add tests for:
// - encoding and then decoding an input object results in the same input object, also when the object contains
//   (recursively) conceiled claims,
// - it uses a more complicated test object than the one below, to hit more features of the encoding/decoding,
// - no _sd or ... are left in the decoded object in cases where they are not expected.
#[cfg(test)]
mod test {
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::Number;
    use serde_json::json;

    use crate::claims::ClaimValue;
    use crate::claims::ObjectClaims;
    use crate::decoder::SdObjectDecoder;
    use crate::encoder::SdObjectEncoder;
    use crate::examples::recursive_disclosures_example;
    use crate::sd_alg::SdAlg;
    use crate::test::array_disclosure;
    use crate::test::object_disclosure;

    #[test]
    fn decode_object_claim_value() {
        let (disclosure_hash, disclosure) = object_disclosure("some_claim", json!("some_value"));
        let (unused_hash, _unused) = object_disclosure("some_claim", json!("some_value"));
        let input = serde_json::from_value::<ClaimValue>(json!({
            "_sd": [&unused_hash, &disclosure_hash],
            "existing_claim": true
        }))
        .unwrap();

        let expected = serde_json::from_value::<ClaimValue>(json!({
            "some_claim": "some_value",
            "existing_claim": true
        }))
        .unwrap();

        let decoded = input
            .decode(&mut IndexMap::from_iter([(disclosure_hash, disclosure)]))
            .unwrap();

        assert_eq!(decoded, expected);
    }

    #[test]
    fn decode_array_claim_value() {
        let (disclosure_hash, disclosure) = array_disclosure(json!("some_value"));
        let (unused_hash, _unused) = array_disclosure(json!("some_value"));
        let input = serde_json::from_value::<ClaimValue>(json!([
            "first_value",
            { "...": &unused_hash },
            { "...": &disclosure_hash },
            "last_value"
        ]))
        .unwrap();

        let expected =
            serde_json::from_value::<ClaimValue>(json!(["first_value", "some_value", "last_value"])).unwrap();

        let decoded = input
            .decode(&mut IndexMap::from_iter([(disclosure_hash, disclosure)]))
            .unwrap();

        assert_eq!(decoded, expected);
    }

    #[rstest]
    #[case(ClaimValue::Null)]
    #[case(ClaimValue::Bool(true))]
    #[case(ClaimValue::Number(Number::from_u128(42).unwrap()))]
    #[case(ClaimValue::String("some".to_string()))]
    fn decode_primitive_claim_values(#[case] value: ClaimValue) {
        let decoded = value.decode(&mut IndexMap::from_iter([])).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn sd_alg() {
        let object = json!({
            "vct": "com.example.pid",
            "iss": "https://issuer.url/",
            "iat": 1683000000,
            "id": "did:value",
            "claim1": [
                "abc"
            ],
            "cnf": {
                "jwk": {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                    "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                }
            }
        });
        let encoder = SdObjectEncoder::try_from(object).unwrap();
        assert_eq!(encoder.clone().encode()._sd_alg, Some(SdAlg::Sha256));
        let decoded = SdObjectDecoder::decode(&encoder.encode(), &IndexMap::new()).unwrap();
        assert!(decoded._sd_alg.is_none());
    }

    #[test]
    fn test_recursive_disclosure() {
        let (claims, disclosure_content) = recursive_disclosures_example();

        let decoded = SdObjectDecoder::decode(&serde_json::from_value(claims).unwrap(), &disclosure_content).unwrap();

        let actual = serde_json::to_value(decoded.claims()).unwrap();

        let expected = json!({
          "address": {
            "country": "DE",
            "locality": "Schulpforta",
            "region": "Sachsen-Anhalt",
            "street_address": "Schulstr. 12"
          },
          "sub": "6c5c0a49-b589-431d-bae7-219122a9ec2c"
        });

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_recursive_disclosure_empty_object() {
        let (claims, disclosure_content) = recursive_disclosures_example();

        // There should be a disclosure value for `address`
        assert!(disclosure_content.into_iter().any(|(k, v)| {
            let disclosure_only_address = IndexMap::from([(k, v)]);

            SdObjectDecoder::decode(
                &serde_json::from_value(claims.clone()).unwrap(),
                &disclosure_only_address,
            )
            .map(|decoded| {
                decoded
                    .claims()
                    .get(&"address".parse().unwrap())
                    .map(|v| matches!(v, ClaimValue::Object(object) if *object == ObjectClaims::default()))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
        }));
    }
}
