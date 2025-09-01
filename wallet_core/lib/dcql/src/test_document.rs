use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use mdoc::test::TestDocument;
use mdoc::test::TestDocuments;
use utils::vec_nonempty;

use crate::ClaimsQuery;
use crate::ClaimsSelection;
use crate::CredentialQuery;
use crate::CredentialQueryFormat;
use crate::Query;
use crate::normalized::FormatCredentialRequest;
use crate::normalized::MdocAttributeRequest;
use crate::normalized::NormalizedCredentialRequest;
use crate::normalized::NormalizedCredentialRequests;

fn credential_request_from((id, source): (usize, TestDocument)) -> NormalizedCredentialRequest {
    let id = format!("id-{id}").try_into().unwrap();

    let claims = source
        .namespaces
        .into_iter()
        .flat_map(|(namespace, attrs)| {
            attrs.into_iter().map(move |entry| MdocAttributeRequest {
                path: vec_nonempty![
                    ClaimPath::SelectByKey(namespace.clone()),
                    ClaimPath::SelectByKey(entry.name),
                ],
                intent_to_retain: None,
            })
        })
        .collect_vec()
        .try_into()
        .expect("TestDocument should have attributes");

    let format_request = FormatCredentialRequest::MsoMdoc {
        doctype_value: source.doc_type,
        claims,
    };

    NormalizedCredentialRequest { id, format_request }
}

impl From<TestDocuments> for NormalizedCredentialRequests {
    fn from(source: TestDocuments) -> Self {
        source
            .0
            .into_iter()
            .enumerate()
            .map(credential_request_from)
            .collect_vec()
            .try_into()
            .expect("TestDocuments should have documents")
    }
}

fn credential_query_from((id, source): (usize, TestDocument)) -> CredentialQuery {
    CredentialQuery {
        id: format!("id-{id}").try_into().unwrap(),
        format: CredentialQueryFormat::MsoMdoc {
            doctype_value: source.doc_type,
        },
        multiple: false,
        trusted_authorities: vec![],
        require_cryptographic_holder_binding: true,
        claims_selection: ClaimsSelection::All {
            claims: source
                .namespaces
                .into_iter()
                .flat_map(|(ns, entries)| {
                    entries.into_iter().map(move |attr| ClaimsQuery {
                        id: None,
                        path: vec_nonempty![ClaimPath::SelectByKey(ns.clone()), ClaimPath::SelectByKey(attr.name)],
                        values: vec![],
                        intent_to_retain: Some(true),
                    })
                })
                .collect_vec()
                .try_into()
                .expect("TestDocument should have attributes"),
        },
    }
}

impl From<TestDocuments> for Query {
    fn from(source: TestDocuments) -> Self {
        let credentials = source
            .0
            .into_iter()
            .enumerate()
            .map(credential_query_from)
            .collect::<Vec<_>>()
            .try_into()
            .expect("TestDocuments should have documents");
        Self {
            credentials,
            credential_sets: vec![],
        }
    }
}
