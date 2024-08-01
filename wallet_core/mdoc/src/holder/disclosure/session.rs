use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use url::Url;

use crate::{
    disclosure::{DeviceResponse, SessionData, SessionStatus},
    errors::Error,
    holder::{DisclosureError, DisclosureResult, HolderError, HttpClient, HttpClientResult},
    identifiers::AttributeIdentifier,
    mdocs::DocType,
    utils::{
        crypto::SessionKey,
        keys::{KeyFactory, MdocEcdsaKey},
    },
    verifier::SessionType,
};

use super::proposed_document::{ProposedDocument, ProposedDocumentAttributes};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierUrlParameters {
    pub session_type: SessionType,
}

pub type ProposedAttributes = IndexMap<DocType, ProposedDocumentAttributes>;

#[derive(Debug)]
pub struct DisclosureMissingAttributes {
    missing_attributes: Vec<AttributeIdentifier>,
}

#[derive(Debug)]
pub struct DisclosureProposal<H, I> {
    return_url: Option<Url>,
    data: CommonDisclosureData<H>,
    device_key: SessionKey,
    proposed_documents: Vec<ProposedDocument<I>>,
}

#[derive(Debug)]
struct CommonDisclosureData<H> {
    client: H,
    verifier_url: Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")] // Symmetrical to `SessionType`.
pub enum DisclosureUriSource {
    Link,
    QrCode,
}

impl DisclosureUriSource {
    pub fn new(is_qr_code: bool) -> Self {
        if is_qr_code {
            Self::QrCode
        } else {
            Self::Link
        }
    }

    /// Returns the expected session type for a source of the received [`ReaderEngagement`].
    pub fn session_type(&self) -> SessionType {
        match self {
            Self::Link => SessionType::SameDevice,
            Self::QrCode => SessionType::CrossDevice,
        }
    }
}

impl DisclosureMissingAttributes {
    pub fn missing_attributes(&self) -> &[AttributeIdentifier] {
        &self.missing_attributes
    }
}

impl<H, I> DisclosureProposal<H, I>
where
    H: HttpClient,
    I: Clone,
{
    pub fn return_url(&self) -> Option<&Url> {
        self.return_url.as_ref()
    }

    pub fn proposed_source_identifiers(&self) -> Vec<&I> {
        self.proposed_documents
            .iter()
            .map(|document| &document.source_identifier)
            .collect()
    }

    pub fn proposed_attributes(&self) -> ProposedAttributes {
        // Get all of the attributes to be disclosed from the
        // prepared `IssuerSigned` on the `ProposedDocument`s.
        self.proposed_documents
            .iter()
            .map(|document| (document.doc_type.clone(), document.proposed_attributes()))
            .collect()
    }

    pub async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<(), Error>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        info!("disclose proposed documents");

        // Clone the proposed documents and construct a `DeviceResponse` by
        // signing these, then encrypt the response with the device key.
        let proposed_documents = self.proposed_documents.to_vec();

        info!("sign proposed documents");

        let device_response = DeviceResponse::from_proposed_documents(proposed_documents, key_factory)
            .await
            .map_err(DisclosureError::before_sharing)?;

        info!("serialize and encrypt device response");

        let session_data = SessionData::serialize_and_encrypt(&device_response, &self.device_key)
            .map_err(DisclosureError::before_sharing)?;

        info!("send device response to verifier");

        // Send the `SessionData` containing the encrypted `DeviceResponse`.
        let response = self.data.send_session_data(&session_data).await?;

        // If we received a `SessionStatus` that is not a
        // termination in the response, return this as an error.
        match response.status {
            Some(status) if status != SessionStatus::Termination => {
                warn!("sending device response failed with status: {status:?}");
                Err(DisclosureError::after_sharing(
                    HolderError::DisclosureResponse(status).into(),
                ))
            }
            _ => {
                info!("sending device response succeeded");
                Ok(())
            }
        }
    }
}

impl<H> CommonDisclosureData<H>
where
    H: HttpClient,
{
    async fn send_session_data(&self, session_data: &SessionData) -> HttpClientResult<SessionData> {
        self.client.post(&self.verifier_url, &session_data).await
    }
}
