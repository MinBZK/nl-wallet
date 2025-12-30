use std::convert::Infallible;

use derive_more::Constructor;

use attestation_data::attributes::Attributes;
use attestation_data::issuable_document::IssuableDocument;
use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use http_utils::urls::BaseUrl;
use openid4vc::issuer::AttributeService;
use openid4vc::oidc;
use openid4vc::token::TokenRequest;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Constructor)]
pub struct MockAttributeService(VecNonEmpty<IssuableDocument>);

pub fn mock_issuable_document_pid() -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(PID_ATTESTATION_TYPE.to_string(), Attributes::nl_pid_example()).unwrap()
}

pub fn mock_issuable_document_address() -> IssuableDocument {
    IssuableDocument::try_new_with_random_id(
        ADDRESS_ATTESTATION_TYPE.to_string(),
        Attributes::nl_pid_address_example(),
    )
    .unwrap()
}

impl Default for MockAttributeService {
    fn default() -> Self {
        Self::new(
            vec![mock_issuable_document_pid(), mock_issuable_document_address()]
                .try_into()
                .unwrap(),
        )
    }
}

impl AttributeService for MockAttributeService {
    type Error = Infallible;

    async fn attributes(&self, _token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let Self(documents) = self;

        // Create a copy of the document having a new random id, ensuring unique batch_ids
        let documents = documents
            .nonempty_iter()
            .map(|document| {
                let (attestation_id, attributes, _) = document.clone().into_parts();
                IssuableDocument::try_new_with_random_id(attestation_id, attributes).unwrap()
            })
            .collect();

        Ok(documents)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config::new_mock(issuer_url))
    }
}
