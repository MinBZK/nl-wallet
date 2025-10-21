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

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests {

    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use serde_json::json;

    use attestation_types::claim_path::ClaimPath;
    use utils::vec_nonempty;

    use crate::builder::SdJwtBuilder;
    use crate::sd_jwt::SdJwtVcClaims;

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
                2,
                [
                    1,
                    "nested_array_value"
                ],
                {
                    "array_object_value": 3,
                }
            ],
            "root_object": {
                "object_value": 4,
                "object_array": [
                    5,
                    [
                        "object_array_value"
                    ],
                    {
                        "nested_object_value": 6,
                    }
                ]
            }
        });

        serde_json::from_value(input_object).unwrap()
    }

    fn by_key(key: &str) -> ClaimPath {
        ClaimPath::SelectByKey(key.to_string())
    }

    fn by_index(index: usize) -> ClaimPath {
        ClaimPath::SelectByIndex(index)
    }

    #[test]
    fn test_encode_decode() {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        let input = test_object();
        let builder = SdJwtBuilder::new(input);

        // conceal all claims, and encode as an SD-JWT
        let sd_jwt = builder
            .make_concealable(vec_nonempty![by_key("root_value")])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_array"), by_index(0)])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_array"), by_index(1)])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_array"), by_index(2)])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_array")])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_object"), by_key("object_value")])
            .unwrap()
            .make_concealable(vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(0)
            ])
            .unwrap()
            .make_concealable(vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(1)
            ])
            .unwrap()
            .make_concealable(vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(2)
            ])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_object"), by_key("object_array")])
            .unwrap()
            .make_concealable(vec_nonempty![by_key("root_object")])
            .unwrap()
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap();

        // disclose all claims
        let verified_sd_jwt = sd_jwt.into_verified();
        let unsigned_sd_jwt = verified_sd_jwt
            .into_presentation_builder()
            .disclose(&vec_nonempty![by_key("root_value")])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_array"), by_index(0)])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_array"), by_index(1)])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_array"), by_index(2)])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_array")])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_object"), by_key("object_value")])
            .unwrap()
            .disclose(&vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(0)
            ])
            .unwrap()
            .disclose(&vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(1)
            ])
            .unwrap()
            .disclose(&vec_nonempty![
                by_key("root_object"),
                by_key("object_array"),
                by_index(2)
            ])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_object"), by_key("object_array")])
            .unwrap()
            .disclose(&vec_nonempty![by_key("root_object")])
            .unwrap()
            .finish();

        // decode the disclosed SD-JWT
        let claims = unsigned_sd_jwt.as_ref().decoded_claims().unwrap();

        assert_eq!(&claims, test_object().claims());
    }
}
