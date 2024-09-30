use wallet_common::keys::factory::{KeyFactory, CredentialEcdsaKey};

use crate::{
    errors::Result,
    iso::disclosure::{DeviceResponse, DeviceResponseVersion},
};

use super::proposed_document::ProposedDocument;

impl DeviceResponse {
    pub async fn from_proposed_documents<I, KF, K>(
        proposed_documents: Vec<ProposedDocument<I>>,
        key_factory: &KF,
    ) -> Result<Self>
    where
        KF: KeyFactory<Key = K>,
        K: CredentialEcdsaKey,
    {
        // Convert all of the `ProposedDocument` entries to `Document` by signing them.
        let documents = ProposedDocument::<I>::sign_multiple(key_factory, proposed_documents).await?;

        // Create a `DeviceResponse` containing the documents.
        let device_response = DeviceResponse {
            version: DeviceResponseVersion::V1_0,
            documents: documents.into(),
            document_errors: None,
            status: 0,
        };

        Ok(device_response)
    }
}
