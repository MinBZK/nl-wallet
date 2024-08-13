use std::{collections::HashSet, iter};

use indexmap::{IndexMap, IndexSet};
use p256::SecretKey;
use url::Url;

use crate::{
    errors::Result,
    examples::{EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
    holder::Mdoc,
    identifiers::AttributeIdentifier,
    iso::{
        device_retrieval::{DocRequest, ItemsRequest, ReaderAuthenticationBytes, ReaderAuthenticationKeyed},
        engagement::{
            ConnectionMethodKeyed, ConnectionMethodType, ConnectionMethodVersion, Engagement, EngagementVersion,
            ReaderEngagement, RestApiOptionsKeyed, SessionTranscript,
        },
    },
    server_keys::KeyPair,
    utils::{
        cose::{self, MdocCose},
        serialization::{CborSeq, TaggedBytes},
    },
};

use super::{proposed_document::ProposedDocument, MdocDataSource, StoredMdoc};

// Constants for testing.
pub const VERIFIER_URL: &str = "http://example.com/disclosure";
pub const RETURN_URL: &str = "http://example.com/return";

// Describe what is in `DeviceResponse::example()`.
pub const EXAMPLE_ATTRIBUTES: [&str; 5] = [
    "family_name",
    "issue_date",
    "expiry_date",
    "document_number",
    "driving_privileges",
];

pub type MdocIdentifier = String;

/// Build an [`ItemsRequest`] from a list of attributes.
pub fn items_request(
    doc_type: String,
    name_space: String,
    attributes: impl Iterator<Item = impl Into<String>>,
) -> ItemsRequest {
    ItemsRequest {
        doc_type,
        name_spaces: IndexMap::from_iter([(
            name_space,
            attributes.map(|attribute| (attribute.into(), false)).collect(),
        )]),
        request_info: None,
    }
}

pub fn example_items_request() -> ItemsRequest {
    items_request(
        EXAMPLE_DOC_TYPE.to_string(),
        EXAMPLE_NAMESPACE.to_string(),
        EXAMPLE_ATTRIBUTES.iter().copied(),
    )
}

pub fn emtpy_items_request() -> ItemsRequest {
    items_request(
        EXAMPLE_DOC_TYPE.to_string(),
        EXAMPLE_NAMESPACE.to_string(),
        iter::empty::<String>(),
    )
}

/// Create a `DocRequest` including reader authentication,
/// based on a `SessionTranscript` and `KeyPair`.
pub async fn create_doc_request(
    items_request: ItemsRequest,
    session_transcript: &SessionTranscript,
    private_key: &KeyPair,
) -> DocRequest {
    // Generate the reader authentication signature, without payload.
    let items_request = items_request.into();
    let reader_auth_keyed = ReaderAuthenticationKeyed::new(session_transcript, &items_request);

    let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
        &TaggedBytes(CborSeq(reader_auth_keyed)),
        cose::new_certificate_header(private_key.certificate()),
        private_key,
        false,
    )
    .await
    .unwrap();
    let reader_auth = Some(cose.0.into());

    // Create and encrypt the `DeviceRequest`.
    DocRequest {
        items_request,
        reader_auth,
    }
}

/// Create `ProposedDocument` based on the example `Mdoc`.
pub fn create_example_proposed_document() -> ProposedDocument<MdocIdentifier> {
    let mdoc = Mdoc::new_example_mock();

    let issuer_certificate = mdoc.issuer_certificate().unwrap();

    ProposedDocument {
        source_identifier: "id_1234".to_string(),
        private_key_id: mdoc.private_key_id,
        doc_type: mdoc.doc_type,
        issuer_signed: mdoc.issuer_signed,
        device_signed_challenge: b"signing_challenge".to_vec(),
        issuer_certificate,
    }
}

/// The `AttributeIdentifier`s contained in the example `Mdoc`.
pub fn example_mdoc_attribute_identifiers() -> IndexSet<AttributeIdentifier> {
    Mdoc::new_example_mock().issuer_signed_attribute_identifiers()
}

/// Create an ordered set of `AttributeIdentifier`s within the
/// example doc type and name space for a given set of attributes.
pub fn example_identifiers_from_attributes(
    attributes: impl IntoIterator<Item = impl Into<String>>,
) -> IndexSet<AttributeIdentifier> {
    attributes
        .into_iter()
        .map(|attribute| AttributeIdentifier {
            doc_type: EXAMPLE_DOC_TYPE.to_string(),
            namespace: EXAMPLE_NAMESPACE.to_string(),
            attribute: attribute.into(),
        })
        .collect()
}

impl ReaderEngagement {
    pub fn try_new(privkey: &SecretKey, verifier_url: Url) -> Result<Self> {
        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: Some(vec![ConnectionMethodKeyed {
                typ: ConnectionMethodType::RestApi,
                version: ConnectionMethodVersion::RestApi,
                connection_options: RestApiOptionsKeyed { uri: verifier_url }.into(),
            }
            .into()]),
            origin_infos: vec![],
        };

        Ok(engagement.into())
    }
}

/// A type that implements `MdocDataSource` and simply returns
/// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
/// `doc_type` is requested.
#[derive(Debug)]
pub struct MockMdocDataSource {
    pub mdocs: Vec<Mdoc>,
    pub has_error: bool,
}
impl MockMdocDataSource {
    pub fn new() -> Self {
        Self {
            mdocs: vec![],
            has_error: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MdocDataSourceError {
    #[error("failed")]
    Failed,
}

impl Default for MockMdocDataSource {
    fn default() -> Self {
        MockMdocDataSource {
            mdocs: vec![Mdoc::new_example_mock()],
            has_error: false,
        }
    }
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = MdocIdentifier;
    type Error = MdocDataSourceError;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        if self.has_error {
            return Err(MdocDataSourceError::Failed);
        }

        let stored_mdocs = self
            .mdocs
            .iter()
            .filter(|mdoc| doc_types.contains(mdoc.doc_type.as_str()))
            .cloned()
            .enumerate()
            .map(|(index, mdoc)| StoredMdoc {
                id: format!("id_{}", index + 1),
                mdoc,
            })
            .collect();

        Ok(vec![stored_mdocs])
    }
}

pub enum ReaderCertificateKind {
    NoReaderRegistration,
    WithReaderRegistration,
}
