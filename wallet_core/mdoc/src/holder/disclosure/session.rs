use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{mdocs::DocType, verifier::SessionType};

use super::proposed_document::ProposedDocumentAttributes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierUrlParameters {
    pub session_type: SessionType,
}

pub type ProposedAttributes = IndexMap<DocType, ProposedDocumentAttributes>;

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
