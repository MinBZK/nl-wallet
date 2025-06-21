use std::collections::HashSet;

use indexmap::IndexMap;
use itertools::Itertools;
use p256::ecdsa::VerifyingKey;

use crate::errors::Result;
use crate::iso::disclosure::IssuerSigned;
use crate::iso::mdocs::Attributes;
use crate::utils::serialization::TaggedBytes;

impl IssuerSigned {
    pub fn public_key(&self) -> Result<VerifyingKey> {
        let public_key = self
            .issuer_auth
            .dangerous_parse_unverified()?
            .0
            .device_key_info
            .try_into()?;
        Ok(public_key)
    }

    pub fn matches_attribute_paths(&self, attribute_paths: &HashSet<(&str, &str)>) -> bool {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                attribute_paths.iter().all(|(name_space, element_id)| {
                    name_spaces
                        .as_ref()
                        .get(*name_space)
                        .map(|attributes| {
                            attributes
                                .as_ref()
                                .iter()
                                .any(|TaggedBytes(signed_item)| signed_item.element_identifier == *element_id)
                        })
                        .unwrap_or_default()
                })
            })
            .unwrap_or_default()
    }

    pub fn into_attribute_subset(self, attribute_paths: &HashSet<(&str, &str)>) -> Self {
        let Some(name_spaces) = self.name_spaces else {
            return self;
        };

        // Remove all of the attributes that are not listed in attribute_paths,
        // which may cause name_spaces to be returned as None.
        let name_spaces = name_spaces
            .into_iter()
            .flat_map(|(name_space, attributes)| {
                let attributes = attributes
                    .into_inner()
                    .into_iter()
                    .filter(|TaggedBytes(signed_item)| {
                        attribute_paths.contains(&(name_space.as_str(), signed_item.element_identifier.as_str()))
                    })
                    .collect_vec();

                // This will return None if the attributes are empty and will subsequently be filtered out.
                Attributes::try_from(attributes)
                    .ok()
                    .map(|attributes| (name_space, attributes))
            })
            .collect::<IndexMap<_, _>>()
            .try_into()
            .ok();

        Self { name_spaces, ..self }
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::mock_remote::MockRemoteEcdsaKey;

    use crate::holder::Mdoc;

    #[tokio::test]
    async fn test_issuer_signed_public_key() {
        let key = SigningKey::random(&mut OsRng);
        let key = MockRemoteEcdsaKey::new("identifier".to_string(), key);
        let mdoc = Mdoc::new_mock_with_key(&key).await;

        let public_key = mdoc
            .issuer_signed
            .public_key()
            .expect("Could not get public key from from IssuerSigned");

        // The example mdoc should contain the generated key.
        assert_eq!(public_key, *key.verifying_key());
    }
}
