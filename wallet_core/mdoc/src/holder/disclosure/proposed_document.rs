use indexmap::{IndexMap, IndexSet};

use crate::{
    errors::Result,
    holder::Mdoc,
    identifiers::AttributeIdentifier,
    iso::{
        basic_sa_ext::Entry,
        disclosure::{DeviceSigned, Document, IssuerSigned},
        mdocs::{DocType, NameSpace},
    },
    utils::keys::{KeyFactory, MdocEcdsaKey},
};

/// This type is derived from an [`Mdoc`] and will be used to construct a [`Document`]
/// for disclosure. Note that this is for internal use of [`DisclosureSession`] only.
#[derive(Debug)]
pub struct ProposedDocument {
    pub private_key_id: String,
    pub doc_type: DocType,
    pub issuer_signed: IssuerSigned,
    pub device_signed_challenge: Vec<u8>,
}

impl ProposedDocument {
    /// For a given set of `Mdoc`s with the same `doc_type`, return two `Vec`s:
    /// * A `Vec<ProposedDocument>` that contains all of the proposed
    ///   disclosure documents that provide all of the required attributes.
    /// * A `Vec<Vec<AttributeIdentifier>>` that contain the missing
    ///   attributes for every `Mdoc` that has at least one attribute missing.
    ///
    /// This means that the sum of the length of these `Vec`s is equal to the
    /// length of the input `Vec<Mdoc>`.
    pub fn candidates_and_missing_attributes_from_mdocs(
        mdocs: Vec<Mdoc>,
        requested_attributes: &IndexSet<AttributeIdentifier>,
        device_signed_challenge: Vec<u8>,
    ) -> (Vec<Self>, Vec<Vec<AttributeIdentifier>>) {
        let mut all_missing_attributes = Vec::new();

        // Collect all `ProposedDocument`s for this `doc_type`,
        // for every `Mdoc` that satisfies the requested attributes.
        let proposed_documents = mdocs
            .into_iter()
            .filter(|mdoc| {
                // Calculate missing attributes for every `Mdoc` and filter it out
                // if we find any. Also, collect the missing attributes separately.
                let available_attributes = mdoc.issuer_signed_attribute_identifiers();
                let missing_attributes = requested_attributes
                    .difference(&available_attributes)
                    .cloned()
                    .collect::<Vec<_>>();

                let is_satisfying = missing_attributes.is_empty();

                if !is_satisfying {
                    all_missing_attributes.push(missing_attributes);
                }

                is_satisfying
            })
            // Convert the matching `Mdoc` to a `ProposedDocument`, based on the requested attributes.
            .map(|mdoc| ProposedDocument::from_mdoc(mdoc, requested_attributes, device_signed_challenge.clone()))
            .collect::<Vec<_>>();

        (proposed_documents, all_missing_attributes)
    }

    /// Create a [`ProposedDocument`] from an [`Mdoc`], containing only those
    /// attributes that are requested and a [`DeviceSigned`] challenge.
    fn from_mdoc(
        mdoc: Mdoc,
        requested_attributes: &IndexSet<AttributeIdentifier>,
        device_signed_challenge: Vec<u8>,
    ) -> Self {
        let name_spaces = mdoc.issuer_signed.name_spaces.map(|name_spaces| {
            name_spaces
                .into_iter()
                .flat_map(|(name_space, attributes)| {
                    let attributes = attributes
                        .0
                        .into_iter()
                        .filter(|attribute| {
                            let attribute_identifier = AttributeIdentifier {
                                doc_type: mdoc.doc_type.clone(),
                                namespace: name_space.clone(),
                                attribute: attribute.0.element_identifier.clone(),
                            };

                            requested_attributes.contains(&attribute_identifier)
                        })
                        .collect::<Vec<_>>();

                    if attributes.is_empty() {
                        return None;
                    }

                    (name_space, attributes.into()).into()
                })
                .collect()
        });

        // Construct everything necessary for signing when the user approves the disclosure.
        let issuer_signed = IssuerSigned {
            name_spaces,
            issuer_auth: mdoc.issuer_signed.issuer_auth,
        };

        ProposedDocument {
            private_key_id: mdoc.private_key_id,
            doc_type: mdoc.doc_type,
            issuer_signed,
            device_signed_challenge,
        }
    }

    /// Return the attributes contained within this [`ProposedDocument`].
    pub fn name_spaces(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .iter()
                    .map(|(name_space, attributes)| (name_space.clone(), attributes.into()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert the [`ProposedDocument`] to a [`Document`] by signing the challenge using the provided `key_factory`.
    #[allow(dead_code)]
    pub async fn sign<'a, K: MdocEcdsaKey + Sync>(
        self,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
    ) -> Result<Document> {
        // Extract the public key from the `IssuerSigned`, construct an existing signing key
        // identifier by `private_key_id` and provide this public key, then use that to sign
        // the saved challenge bytes asynchronously.
        let public_key = self.issuer_signed.public_key()?;
        let private_key = key_factory.generate_existing(&self.private_key_id, public_key);
        let device_signed = DeviceSigned::new_signature(&private_key, &self.device_signed_challenge).await?;

        let document = Document {
            doc_type: self.doc_type,
            issuer_signed: self.issuer_signed,
            device_signed,
            errors: None,
        };

        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use coset::Header;
    use wallet_common::keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier};

    use crate::{
        errors::Error,
        iso::disclosure::DeviceAuth,
        mock::{FactorySoftwareEcdsaKeyError, SoftwareKeyFactory},
        utils::{
            cose::{self, CoseError},
            serialization::TaggedBytes,
        },
    };

    use super::{super::tests::*, *};

    #[test]
    fn test_proposed_document_from_mdoc() {
        let mdoc = create_example_mdoc();
        let doc_type = mdoc.doc_type.clone();
        let private_key_id = mdoc.private_key_id.clone();
        let issuer_auth = mdoc.issuer_signed.issuer_auth.clone();

        let requested_attributes = vec![
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/driving_privileges",
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/family_name",
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/document_number",
        ]
        .into_iter()
        .map(|attribute| attribute.parse().unwrap())
        .collect();

        let proposed_document = ProposedDocument::from_mdoc(mdoc, &requested_attributes, b"foobar".to_vec());

        assert_eq!(proposed_document.doc_type, doc_type);
        assert_eq!(proposed_document.private_key_id, private_key_id);
        assert_eq!(proposed_document.device_signed_challenge, b"foobar");

        let attributes_identifiers = proposed_document
            .issuer_signed
            .name_spaces
            .as_ref()
            .and_then(|name_spaces| name_spaces.get("org.iso.18013.5.1"))
            .map(|attributes| {
                attributes
                    .0
                    .iter()
                    .map(|attribute| attribute.0.element_identifier.as_str())
                    .collect::<Vec<_>>()
            })
            .expect("Could not get expected attributes from proposed document");

        assert_eq!(
            attributes_identifiers,
            ["family_name", "document_number", "driving_privileges"]
        );
        assert_eq!(proposed_document.issuer_signed.issuer_auth, issuer_auth);
    }

    #[test]
    fn test_proposed_document_candidates_and_missing_attributes_from_mdocs() {
        let mdoc1 = create_example_mdoc();
        let mdoc2 = {
            let mut mdoc = mdoc1.clone();
            let attributes = &mut mdoc
                .issuer_signed
                .name_spaces
                .as_mut()
                .unwrap()
                .get_mut("org.iso.18013.5.1")
                .unwrap()
                .0;

            // Remove `issue_date` and `expiry_date`.
            attributes.remove(1);
            attributes.remove(1);

            mdoc
        };
        let mdoc3 = mdoc1.clone();
        let mdoc4 = {
            let mut mdoc = mdoc1.clone();
            let attributes = &mut mdoc
                .issuer_signed
                .name_spaces
                .as_mut()
                .unwrap()
                .get_mut("org.iso.18013.5.1")
                .unwrap()
                .0;

            attributes.clear();

            mdoc
        };

        let doc_type = mdoc1.doc_type.clone();
        let private_key_id = mdoc1.private_key_id.clone();

        let requested_attributes = vec![
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/driving_privileges",
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/issue_date",
            "org.iso.18013.5.1.mDL/org.iso.18013.5.1/expiry_date",
        ]
        .into_iter()
        .map(|attribute| attribute.parse().unwrap())
        .collect();

        let (proposed_documents, missing_attributes) = ProposedDocument::candidates_and_missing_attributes_from_mdocs(
            vec![mdoc1, mdoc2, mdoc3, mdoc4],
            &requested_attributes,
            b"challenge".to_vec(),
        );

        assert_eq!(proposed_documents.len(), 2);
        proposed_documents.into_iter().for_each(|proposed_document| {
            assert_eq!(proposed_document.doc_type, doc_type);
            assert_eq!(proposed_document.private_key_id, private_key_id);
            assert_eq!(
                proposed_document
                    .issuer_signed
                    .name_spaces
                    .unwrap()
                    .get("org.iso.18013.5.1")
                    .unwrap()
                    .0
                    .len(),
                3
            );
        });

        assert_eq!(missing_attributes.len(), 2);
        assert_eq!(
            missing_attributes[0]
                .iter()
                .map(|attribute| attribute.attribute.as_str())
                .collect::<Vec<_>>(),
            ["issue_date", "expiry_date"]
        );
        assert_eq!(
            missing_attributes[1]
                .iter()
                .map(|attribute| attribute.attribute.as_str())
                .collect::<Vec<_>>(),
            ["driving_privileges", "issue_date", "expiry_date"]
        );
    }

    #[tokio::test]
    async fn test_proposed_document_sign() {
        // Create a `ProposedDocument` from the example `Mdoc`.
        let proposed_document = create_example_proposed_document();

        // Collect all of the expected values.
        let expected_doc_type = proposed_document.doc_type.clone();
        let expected_issuer_signed = proposed_document.issuer_signed.clone();

        let key = SoftwareEcdsaKey::new(&proposed_document.private_key_id);
        let expected_cose = cose::sign_cose(
            &proposed_document.device_signed_challenge,
            Header::default(),
            &key,
            false,
        )
        .await
        .unwrap();

        // Conversion to `Document` by signing should succeed.
        let document = proposed_document
            .sign(&SoftwareKeyFactory::default())
            .await
            .expect("Could not sign ProposedDocument");

        // Test all of the expected values, including the `DeviceSigned` COSE signature.
        assert_eq!(document.doc_type, expected_doc_type);
        assert_eq!(document.issuer_signed, expected_issuer_signed);
        assert_matches!(document.device_signed, DeviceSigned {
            name_spaces: TaggedBytes(name_spaces),
            device_auth: DeviceAuth::DeviceSignature(mdoc_cose)
        } if name_spaces.is_empty() && mdoc_cose.0 == expected_cose);
        assert!(document.errors.is_none());
    }

    #[tokio::test]
    async fn test_proposed_document_sign_error() {
        // Set up a `KeyFactory` that returns keys that fail at signing.
        let proposed_document = create_example_proposed_document();
        let key_factory = SoftwareKeyFactory {
            has_generating_error: false,
            has_key_signing_error: true,
        };

        // Conversion to `Document` should simply forward the signing error.
        let error = proposed_document
            .sign(&key_factory)
            .await
            .expect_err("Signing ProposedDocument should have resulted in an error");

        assert_matches!(error, Error::Cose(
            CoseError::Signing(signing_error)
        ) if signing_error.is::<FactorySoftwareEcdsaKeyError>());
    }
}
