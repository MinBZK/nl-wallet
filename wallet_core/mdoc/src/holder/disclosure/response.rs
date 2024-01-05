use futures::future;

use crate::{
    errors::Result,
    iso::disclosure::{DeviceResponse, DeviceResponseVersion},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};

use super::proposed_document::ProposedDocument;

impl DeviceResponse {
    pub(super) async fn from_proposed_documents<I, KF, K>(
        proposed_documents: Vec<ProposedDocument<I>>,
        key_factory: &KF,
    ) -> Result<Self>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        // Convert all of the `ProposedDocument` entries to `Document` by signing them.
        // TODO: Do this in bulk, as this will be serialized by the implementation.
        let documents = future::try_join_all(
            proposed_documents
                .into_iter()
                .map(|proposed_document| proposed_document.sign(key_factory)),
        )
        .await?;

        // Create a `DeviceResponse` containing the documents.
        let device_response = DeviceResponse {
            version: DeviceResponseVersion::V1_0,
            documents: documents.into(),
            document_errors: None, // TODO: Consider using this for reporting errors per mdoc
            status: 0,
        };

        Ok(device_response)
    }
}
