use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;

use crate::token::CredentialPreview;
use crate::token::CredentialPreviewContent;
use crate::wallet_issuance::WalletIssuanceError;

#[derive(Debug, Clone)]
pub struct NormalizedCredentialPreview {
    pub content: CredentialPreviewContent,

    pub normalized_metadata: NormalizedTypeMetadata,

    pub raw_metadata: SortedTypeMetadataDocuments,
}

impl NormalizedCredentialPreview {
    pub fn try_new(preview: CredentialPreview) -> Result<Self, WalletIssuanceError> {
        let (normalized_metadata, raw_metadata) = preview
            .type_metadata
            .into_normalized(&preview.content.credential_payload.attestation_type)?;
        preview
            .content
            .credential_payload
            .attributes
            .validate(&normalized_metadata)?;

        Ok(Self {
            content: preview.content,
            normalized_metadata,
            raw_metadata,
        })
    }
}
