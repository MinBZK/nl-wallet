use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
};

use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    holder::{HttpClient, HttpClientResult, Mdoc, MdocCopies, MdocDataSource, StoredMdoc},
    iso::mdocs::DocType,
    server_state::MemorySessionStore,
    utils::serialization::{self},
    verifier::{DisclosureData, Verifier},
};

type MockVerifier = Verifier<MemorySessionStore<DisclosureData>>;

struct MockDisclosureHttpClient {
    verifier: Arc<MockVerifier>,
}

impl MockDisclosureHttpClient {
    pub fn new(verifier: Arc<MockVerifier>) -> Self {
        MockDisclosureHttpClient { verifier }
    }
}

impl HttpClient for MockDisclosureHttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> HttpClientResult<R>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let session_token = url.path_segments().unwrap().last().unwrap().to_string();
        let msg = serialization::cbor_serialize(val).unwrap();

        let session_data = self
            .verifier
            .process_message(&msg, &session_token.into(), url.clone())
            .await
            .unwrap();

        let response_msg = serialization::cbor_serialize(&session_data).unwrap();
        let response = serialization::cbor_deserialize(response_msg.as_slice()).unwrap();

        Ok(response)
    }
}

struct MockMdocDataSource(HashMap<DocType, MdocCopies>);

impl From<Vec<Mdoc>> for MockMdocDataSource {
    fn from(value: Vec<Mdoc>) -> Self {
        MockMdocDataSource(
            value
                .into_iter()
                .map(|mdoc| (mdoc.doc_type.clone(), vec![mdoc].into()))
                .collect(),
        )
    }
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = String;
    type Error = Infallible;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        let stored_mdocs = self
            .0
            .iter()
            .filter_map(|(doc_type, mdoc_copies)| {
                if doc_types.contains(doc_type.as_str()) {
                    return vec![StoredMdoc {
                        id: format!("{}_id", doc_type.clone()),
                        mdoc: mdoc_copies.cred_copies.first().unwrap().clone(),
                    }]
                    .into();
                }

                None
            })
            .collect();

        Ok(stored_mdocs)
    }
}
