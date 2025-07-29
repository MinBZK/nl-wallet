// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use indexmap::IndexMap;
use serde_json::Map;
use serde_json::Value;

use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
use crate::encoder::SD_ALG;
use crate::error::Error;

const RESERVED_CLAIM_NAMES: &[&str] = &["_sd", "..."];

/// Substitutes digests in an SD-JWT object by their corresponding plain text values provided by disclosures.
pub struct SdObjectDecoder;

impl SdObjectDecoder {
    /// Decodes an SD-JWT `object` containing by Substituting the digests with their corresponding
    /// plain text values provided by `disclosures`.
    pub fn decode(
        &self,
        object: &Map<String, Value>,
        disclosures: &IndexMap<String, Disclosure>,
    ) -> Result<Map<String, Value>, Error> {
        // `processed_digests` are kept track of in case one digest appears more than once which
        // renders the SD-JWT invalid.
        let mut processed_digests: Vec<String> = vec![];

        // Decode the object recursively.
        let mut decoded = self.decode_object(object, disclosures, &mut processed_digests)?;

        if processed_digests.len() != disclosures.len() {
            return Err(Error::UnusedDisclosures(
                disclosures.len().saturating_sub(processed_digests.len()),
            ));
        }

        // Remove `_sd_alg` in case it exists.
        decoded.remove(SD_ALG);

        Ok(decoded)
    }

    fn decode_object(
        &self,
        object: &Map<String, Value>,
        disclosures: &IndexMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
    ) -> Result<Map<String, Value>, Error> {
        let mut output: Map<String, Value> = Map::new();
        for (key, value) in object {
            match value {
                Value::Object(object) => {
                    let decoded_object = self.decode_object(object, disclosures, processed_digests)?;
                    output.insert(key.to_string(), Value::Object(decoded_object));
                }
                Value::Array(sd_array) if key == DIGESTS_KEY => {
                    for digest in sd_array {
                        if let Some((DisclosureContent::ObjectProperty(_, claim_name, _), decoded_value)) = self
                            .disclosure_and_decoded_value_for_array_value(
                                digest,
                                disclosures,
                                processed_digests,
                                |disclosure| Self::verify_disclosure_for_object(disclosure, &output),
                            )?
                        {
                            output.insert(claim_name.clone(), decoded_value);
                        }
                    }
                }
                Value::Array(array) => {
                    let decoded_array = self.decode_array(array, disclosures, processed_digests)?;
                    output.insert(key.to_string(), Value::Array(decoded_array));
                }
                _ => {
                    output.insert(key.to_string(), value.clone());
                }
            }
        }
        Ok(output)
    }

    fn decode_array(
        &self,
        array: &[Value],
        disclosures: &IndexMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
    ) -> Result<Vec<Value>, Error> {
        let mut output: Vec<Value> = vec![];
        for value in array {
            match value {
                Value::Object(object) => {
                    self.decode_array_nested_object(object, disclosures, processed_digests, &mut output)?;
                }
                Value::Array(array) => {
                    // Nested arrays need to be decoded too.
                    let decoded = self.decode_array(array, disclosures, processed_digests)?;
                    output.push(Value::Array(decoded));
                }
                _ => {
                    // Append the rest of the values.
                    output.push(value.clone());
                }
            }
        }

        Ok(output)
    }

    fn decode_array_nested_object(
        &self,
        object: &serde_json::Map<String, serde_json::Value>,
        disclosures: &IndexMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
        output: &mut Vec<Value>,
    ) -> Result<(), Error> {
        for (key, value) in object {
            if key == ARRAY_DIGEST_KEY {
                if object.keys().len() != 1 {
                    return Err(Error::InvalidArrayDisclosureObject);
                }

                if let Some((_, decoded_value)) = self.disclosure_and_decoded_value_for_array_value(
                    value,
                    disclosures,
                    processed_digests,
                    |disclosure| match disclosure.content {
                        DisclosureContent::ObjectProperty(_, _, _) => {
                            Err(Error::InvalidDisclosure("array length must be 2".to_string()))
                        }
                        _ => Ok(()),
                    },
                )? {
                    output.push(decoded_value);
                }
            } else {
                let decoded_object = self.decode_object(object, disclosures, processed_digests)?;
                output.push(Value::Object(decoded_object));
                break;
            }
        }

        Ok(())
    }

    fn disclosure_and_decoded_value_for_array_value<'a>(
        &self,
        digest: &Value,
        disclosures: &'a IndexMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
        verify_disclosure: impl Fn(&Disclosure) -> Result<(), Error>,
    ) -> Result<Option<(&'a DisclosureContent, Value)>, Error> {
        let digest_str = digest
            .as_str()
            .ok_or(Error::DataTypeMismatch(format!("{digest} is not a string")))?
            .to_string();

        // Reject if any digests were found more than once.
        if processed_digests.contains(&digest_str) {
            return Err(Error::DuplicateDigest(digest_str));
        }

        // Check if a disclosure of this digest is available
        // and return it and the decoded value
        if let Some(disclosure) = disclosures.get(&digest_str) {
            verify_disclosure(disclosure)?;

            processed_digests.push(digest_str.clone());

            let recursively_decoded = self.decode_claim_value(disclosure, disclosures, processed_digests)?;
            return Ok(Some((&disclosure.content, recursively_decoded)));
        }

        Ok(None)
    }

    fn verify_disclosure_for_object(disclosure: &Disclosure, output: &Map<String, Value>) -> Result<(), Error> {
        let claim_name = match &disclosure.content {
            DisclosureContent::ObjectProperty(_, claim_name, _) => Ok(claim_name),
            _ => Err(Error::DataTypeMismatch(format!("disclosure type error: {disclosure}"))),
        }?;

        if RESERVED_CLAIM_NAMES.contains(&claim_name.as_str()) {
            return Err(Error::ReservedClaimNameUsed(claim_name.clone()));
        }

        if output.contains_key(claim_name) {
            return Err(Error::ClaimCollision(claim_name.clone()));
        }

        Ok(())
    }

    fn decode_claim_value(
        &self,
        disclosure: &Disclosure,
        disclosures: &IndexMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
    ) -> Result<Value, Error> {
        let decoded = match disclosure.claim_value() {
            Value::Array(sub_arr) => Value::Array(self.decode_array(sub_arr, disclosures, processed_digests)?),
            Value::Object(sub_obj) => Value::Object(self.decode_object(sub_obj, disclosures, processed_digests)?),
            _ => disclosure.claim_value().clone(),
        };

        Ok(decoded)
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
    use serde_json::json;

    use crate::decoder::SdObjectDecoder;
    use crate::encoder::SdObjectEncoder;
    use crate::examples::recursive_disclosures_example;

    #[test]
    fn sd_alg() {
        let object = json!({
          "id": "did:value",
          "claim1": [
            "abc"
          ],
        });
        let mut encoder = SdObjectEncoder::try_from(object).unwrap();
        encoder.add_sd_alg_property();
        assert_eq!(encoder.clone().encode().get("_sd_alg").unwrap(), "sha-256");
        let decoder = SdObjectDecoder;
        let decoded = decoder
            .decode(encoder.encode().as_object().unwrap(), &IndexMap::new())
            .unwrap();
        assert!(decoded.get("_sd_alg").is_none());
    }

    #[test]
    fn test_recursive_disclosure() {
        let (claims, disclosure_content) = recursive_disclosures_example();

        let decoded = SdObjectDecoder
            .decode(claims.as_object().unwrap(), &disclosure_content)
            .unwrap();

        let actual = serde_json::to_value(&decoded).unwrap();

        let expected = json!({
          "address": {
            "country": "DE",
            "locality": "Schulpforta",
            "region": "Sachsen-Anhalt",
            "street_address": "Schulstr. 12"
          },
          "exp": 1883000000,
          "iat": 1683000000,
          "iss": "https://issuer.example.com",
          "sub": "6c5c0a49-b589-431d-bae7-219122a9ec2c"
        });

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_recursive_disclosure_empty_object() {
        let (claims, mut disclosure_content) = recursive_disclosures_example();
        // Take the last disclosure value, which is `address`
        let disclosure_only_address = IndexMap::from([disclosure_content.pop().unwrap()]);

        let decoded = SdObjectDecoder
            .decode(claims.as_object().unwrap(), &disclosure_only_address)
            .unwrap();

        assert_eq!(decoded.get("address").unwrap().to_string(), "{}");
    }
}
