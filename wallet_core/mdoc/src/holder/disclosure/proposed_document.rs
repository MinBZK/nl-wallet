use indexmap::{IndexMap, IndexSet};

use crate::{
    holder::Mdoc,
    identifiers::AttributeIdentifier,
    iso::{
        basic_sa_ext::Entry,
        disclosure::IssuerSigned,
        mdocs::{DocType, NameSpace},
    },
};

/// This type is derived from an [`Mdoc`] and will be used to construct a [`Document`]
/// for disclosure. Note that this is for internal use of [`DisclosureSession`] only.
#[allow(dead_code)]
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
}

#[cfg(test)]
mod tests {
    use crate::{examples::Examples, mock};

    use super::*;

    #[test]
    fn test_proposed_document_from_mdoc() {
        let trust_anchors = Examples::iaca_trust_anchors();
        let mdoc = mock::mdoc_from_example_device_response(trust_anchors);

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
        let trust_anchors = Examples::iaca_trust_anchors();

        let mdoc1 = mock::mdoc_from_example_device_response(trust_anchors);
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
}
