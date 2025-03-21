// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde_json::Map;
use serde_json::Value;

use crate::disclosure::Disclosure;
use crate::encoder::ARRAY_DIGEST_KEY;
use crate::encoder::DIGESTS_KEY;
use crate::encoder::SD_ALG;
use crate::error::Error;

/// Substitutes digests in an SD-JWT object by their corresponding plain text values provided by disclosures.
pub struct SdObjectDecoder;

impl SdObjectDecoder {
    /// Decodes an SD-JWT `object` containing by Substituting the digests with their corresponding
    /// plain text values provided by `disclosures`.
    pub fn decode(
        &self,
        object: &Map<String, Value>,
        disclosures: &HashMap<String, Disclosure>,
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
        disclosures: &HashMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
    ) -> Result<Map<String, Value>, Error> {
        let mut output: Map<String, Value> = object.clone();
        for (key, value) in object.iter() {
            match value {
                Value::Array(sd_array) if key == DIGESTS_KEY => {
                    for digest in sd_array {
                        let digest_str = digest
                            .as_str()
                            .ok_or(Error::DataTypeMismatch(format!("{} is not a string", digest)))?
                            .to_string();

                        // Reject if any digests were found more than once.
                        if processed_digests.contains(&digest_str) {
                            return Err(Error::DuplicateDigestError(digest_str));
                        }

                        // Check if a disclosure of this digest is available
                        // and insert its claim name and value in the object.
                        if let Some(disclosure) = disclosures.get(&digest_str) {
                            let claim_name = disclosure.claim_name.clone().ok_or(Error::DataTypeMismatch(format!(
                                "disclosure type error: {}",
                                disclosure
                            )))?;

                            if output.contains_key(&claim_name) {
                                return Err(Error::ClaimCollisionError(claim_name));
                            }
                            processed_digests.push(digest_str.clone());

                            let recursively_decoded = match disclosure.claim_value {
                                Value::Array(ref sub_arr) => {
                                    Value::Array(self.decode_array(sub_arr, disclosures, processed_digests)?)
                                }
                                Value::Object(ref sub_obj) => {
                                    Value::Object(self.decode_object(sub_obj, disclosures, processed_digests)?)
                                }
                                _ => disclosure.claim_value.clone(),
                            };

                            output.insert(claim_name, recursively_decoded);
                        }
                    }
                    if output
                        .get(DIGESTS_KEY)
                        .expect("output has a `DIGEST_KEY` property")
                        .is_array()
                    {
                        output.remove(DIGESTS_KEY);
                    }
                }
                Value::Object(object) => {
                    let decoded_object = self.decode_object(object, disclosures, processed_digests)?;
                    if !decoded_object.is_empty() {
                        output.insert(key.to_string(), Value::Object(decoded_object));
                    }
                }
                Value::Array(array) => {
                    let decoded_array = self.decode_array(array, disclosures, processed_digests)?;
                    if !decoded_array.is_empty() {
                        output.insert(key.to_string(), Value::Array(decoded_array));
                    }
                }
                // Only objects and arrays require decoding.
                _ => {}
            }
        }
        Ok(output)
    }

    fn decode_array(
        &self,
        array: &[Value],
        disclosures: &HashMap<String, Disclosure>,
        processed_digests: &mut Vec<String>,
    ) -> Result<Vec<Value>, Error> {
        let mut output: Vec<Value> = vec![];
        for value in array.iter() {
            if let Some(object) = value.as_object() {
                for (key, value) in object.iter() {
                    if key == ARRAY_DIGEST_KEY {
                        if object.keys().len() != 1 {
                            return Err(Error::InvalidArrayDisclosureObject);
                        }

                        let digest_in_array = value
                            .as_str()
                            .ok_or(Error::DataTypeMismatch(format!("{} is not a string", key)))?
                            .to_string();

                        // Reject if any digests were found more than once.
                        if processed_digests.contains(&digest_in_array) {
                            return Err(Error::DuplicateDigestError(digest_in_array));
                        }
                        if let Some(disclosure) = disclosures.get(&digest_in_array) {
                            if disclosure.claim_name.is_some() {
                                return Err(Error::InvalidDisclosure("array length must be 2".to_string()));
                            }
                            processed_digests.push(digest_in_array.clone());
                            // Recursively decoded the disclosed values.
                            let recursively_decoded = match disclosure.claim_value {
                                Value::Array(ref sub_arr) => {
                                    Value::Array(self.decode_array(sub_arr, disclosures, processed_digests)?)
                                }
                                Value::Object(ref sub_obj) => {
                                    Value::Object(self.decode_object(sub_obj, disclosures, processed_digests)?)
                                }
                                _ => disclosure.claim_value.clone(),
                            };

                            output.push(recursively_decoded);
                        }
                    } else {
                        let decoded_object = self.decode_object(object, disclosures, processed_digests)?;
                        output.push(Value::Object(decoded_object));
                        break;
                    }
                }
            } else if let Some(arr) = value.as_array() {
                // Nested arrays need to be decoded too.
                let decoded = self.decode_array(arr, disclosures, processed_digests)?;
                output.push(Value::Array(decoded));
            } else {
                // Append the rest of the values.
                output.push(value.clone());
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::decoder::SdObjectDecoder;
    use crate::encoder::SdObjectEncoder;

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
        assert_eq!(encoder.object.get("_sd_alg").unwrap(), "sha-256");
        let decoder = SdObjectDecoder;
        let decoded = decoder
            .decode(encoder.object.as_object().unwrap(), &HashMap::new())
            .unwrap();
        assert!(decoded.get("_sd_alg").is_none());
    }
}
