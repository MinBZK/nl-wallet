use itertools::Itertools;

use dcql::UniqueIdVec;
use dcql::normalized::NormalizedCredentialRequests;
use mdoc::test::TestDocument;
use mdoc::test::TestDocuments;

use crate::disclosure::DisclosedAttestations;
use crate::disclosure::DisclosedAttributes;

pub fn test_documents_assert_matches_disclosed_attestations(
    test_documents: &TestDocuments,
    disclosed_attestations: &UniqueIdVec<DisclosedAttestations>,
) {
    // Verify the number of responses.
    assert_eq!(disclosed_attestations.len().get(), test_documents.len());

    let requests = NormalizedCredentialRequests::from(test_documents.clone());

    let TestDocuments(documents) = &test_documents;
    for (
        TestDocument {
            doc_type: expected_doc_type,
            issuer_uri: expected_issuer,
            namespaces: expected_namespaces,
            ..
        },
        request,
    ) in documents.iter().zip_eq(requests.as_ref())
    {
        let attestations = &disclosed_attestations
            .as_ref()
            .iter()
            .filter(|attestations| attestations.id == request.id)
            .exactly_one()
            .expect("disclosed attestations should include credential query identifier")
            .attestations;

        // The response should contain exactly one attestation.
        assert_eq!(attestations.len().get(), 1);

        let attestation = attestations.first();

        // Verify the attestation type.
        assert_eq!(attestation.attestation_type, *expected_doc_type);

        // Verify the issuer.
        assert_eq!(attestation.issuer_uri, *expected_issuer);

        // Verify the actual attributes.
        let DisclosedAttributes::MsoMdoc(attributes) = &attestation.attributes else {
            panic!("disclosed attributes should be in mdoc format");
        };

        assert_eq!(attributes.len(), expected_namespaces.len());

        for (expected_namespace, expected_entries) in expected_namespaces {
            let disclosed_namespace = attributes
                .get(expected_namespace)
                .expect("disclosed attributes should contain namespace");

            assert_eq!(disclosed_namespace.len(), expected_entries.len());

            for expected_entry in expected_entries {
                let attribute = disclosed_namespace
                    .get(&expected_entry.name)
                    .expect("disclosed attributes should contain entry");

                assert_eq!(ciborium::Value::from(attribute.clone()), expected_entry.value);
            }
        }
    }
}
