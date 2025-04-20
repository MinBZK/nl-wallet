use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use mdoc::verifier::DocumentDisclosedAttributes;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::Created;
use openid4vc::issuer::IssuanceData;
use openid4vc::server_state::SessionState;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosureResultHandler;
use openid4vc::verifier::DisclosureResultHandlerError;
use openid4vc::verifier::PostAuthResponseError;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum AttributesFetcherError {
    #[error("unknown usecase: {0}")]
    UnknownUsecase(String),
    #[error("failed to fetch attributes to be issued: {0}")]
    AttestationsFetching(#[from] reqwest::Error),
    #[error("failed to deserialize attributes to be issued: {0}")]
    AttestationsDeserializing(#[from] serde_json::Error),
}

/// Represents types that can take disclosed attributes and respond with attestations to be issued.
#[trait_variant::make(Send)]
pub trait AttributesFetcher {
    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<Vec<IssuableDocument>, AttributesFetcherError>;
}

pub struct HttpAttributesFetcher {
    pub urls: HashMap<String, BaseUrl>,
}

impl AttributesFetcher for HttpAttributesFetcher {
    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<Vec<IssuableDocument>, AttributesFetcherError> {
        let usecase_url = self
            .urls
            .get(usecase_id)
            .ok_or_else(|| AttributesFetcherError::UnknownUsecase(usecase_id.to_string()))?
            .as_ref()
            .clone();

        let to_issue = default_reqwest_client_builder()
            .build()
            .expect("failed to construct reqwest instance")
            .post(usecase_url)
            .json(disclosed)
            .send()
            .await?
            .json()
            .await?;

        Ok(to_issue)
    }
}

/// Receives disclosed attributes, exchanges those for attestations to be issued, and creates a new issuance session
/// by implementing [`DisclosureResultHandler`].
pub struct IssuanceResultHandler<IS, A> {
    pub attributes_fetcher: A,
    pub issuance_sessions: Arc<IS>,
    pub credential_issuer: BaseUrl,
}

#[derive(Debug, thiserror::Error)]
pub enum IssuanceResultHandlerError {
    #[error("failed to fetch attributes: {0}")]
    FetchingAttributes(#[from] AttributesFetcherError),
    #[error("failed to write issuance session: {0}")]
    WriteIssuanceSession(#[from] SessionStoreError),
}

impl<IS, A> DisclosureResultHandler for IssuanceResultHandler<IS, A>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
{
    async fn disclosure_result(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<HashMap<String, String>, DisclosureResultHandlerError> {
        let to_issue = self
            .attributes_fetcher
            .attributes(usecase_id, disclosed)
            .await
            .map_err(|err| DisclosureResultHandlerError::Other(err.into()))?;

        // Return a specific error code if there are no attestations to be issued so the wallet
        // can distinguish this case from other (error) cases.
        let to_issue: VecNonEmpty<_> = to_issue
            .try_into()
            .map_err(|_| DisclosureResultHandlerError::WalletError(PostAuthResponseError::NoIssuableAttestations))?;

        let credential_configuration_ids = to_issue
            .iter()
            .map(|attestation| attestation.attestation_type().to_string())
            .collect();

        // Start a new issuance session.
        let token = SessionToken::new_random();
        let session = SessionState::new(
            token.clone(),
            IssuanceData::Created(Created {
                issuable_documents: Some(to_issue),
            }),
        );
        self.issuance_sessions
            .write(session, true)
            .await
            .map_err(|err| DisclosureResultHandlerError::Other(err.into()))?;

        let credential_offer = CredentialOfferContainer {
            credential_offer: CredentialOffer {
                credential_issuer: self.credential_issuer.clone(),
                credential_configuration_ids,
                grants: Some(Grants::PreAuthorizedCode {
                    pre_authorized_code: GrantPreAuthorizedCode::new(token.as_ref().clone().into()),
                }),
            },
        };

        // If `serde_urlencoded` would have something like `serde_json::Value` or `ciborium::Value`,
        // then this would be a lot less awkward.
        let query_params = serde_urlencoded::from_str(
            &serde_urlencoded::to_string(credential_offer)
                .map_err(|err| DisclosureResultHandlerError::Other(err.into()))?,
        )
        .map_err(|err| DisclosureResultHandlerError::Other(err.into()))?;

        Ok(query_params)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use indexmap::IndexMap;

    use mdoc::verifier::DocumentDisclosedAttributes;
    use openid4vc::issuable_document::IssuableDocument;

    use super::AttributesFetcher;
    use super::AttributesFetcherError;

    pub struct MockAttributesFetcher(pub Vec<IssuableDocument>);

    impl AttributesFetcher for MockAttributesFetcher {
        async fn attributes(
            &self,
            _usecase_id: &str,
            _disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<Vec<IssuableDocument>, AttributesFetcherError> {
            Ok(self.0.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;

    use mdoc::verifier::DocumentDisclosedAttributes;
    use mdoc::Tdate;
    use mdoc::ValidityInfo;
    use openid4vc::attributes::Attribute;
    use openid4vc::attributes::AttributeValue;
    use openid4vc::credential::CredentialOffer;
    use openid4vc::issuable_document::IssuableDocument;
    use openid4vc::issuer::IssuanceData;
    use openid4vc::server_state::MemorySessionStore;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionStoreTimeouts;
    use openid4vc::server_state::SessionToken;
    use openid4vc::verifier::DisclosureResultHandler;
    use openid4vc::verifier::DisclosureResultHandlerError;
    use openid4vc::verifier::PostAuthResponseError;

    use crate::disclosure::mock::MockAttributesFetcher;

    use super::AttributesFetcher;
    use super::AttributesFetcherError;
    use super::IssuanceResultHandler;

    pub struct TestAttributesFetcher;

    fn mock_disclosed_attrs(attestation_type: String) -> IndexMap<String, DocumentDisclosedAttributes> {
        IndexMap::from([(
            attestation_type,
            DocumentDisclosedAttributes {
                attributes: IndexMap::new(),
                issuer_uri: "https://example.com".parse().unwrap(),
                ca: "ca".to_string(),
                validity_info: ValidityInfo {
                    signed: Tdate::now(),
                    valid_from: Tdate::now(),
                    valid_until: Tdate::now(),
                    expected_update: None,
                },
            },
        )])
    }

    impl AttributesFetcher for TestAttributesFetcher {
        async fn attributes(
            &self,
            _usecase_id: &str,
            disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<Vec<IssuableDocument>, AttributesFetcherError> {
            // Insert the received attribute type into the issuable document to demonstrate that the
            // issued attributes can depend on the disclosed attributes.
            let (attestation_type, _) = disclosed.first().unwrap();

            Ok(vec![IssuableDocument::try_new(
                attestation_type.clone(),
                IndexMap::from([(
                    "attr_name".to_string(),
                    Attribute::Single(AttributeValue::Text("attrvalue".to_string())),
                )]),
            )
            .unwrap()])
        }
    }

    #[tokio::test]
    async fn it_works() {
        let issuance_sessions: Arc<MemorySessionStore<IssuanceData>> =
            Arc::new(MemorySessionStore::new(SessionStoreTimeouts::default()));

        let result_handler = IssuanceResultHandler {
            attributes_fetcher: TestAttributesFetcher,
            issuance_sessions: Arc::clone(&issuance_sessions),
            credential_issuer: "https://example.com".parse().unwrap(),
        };

        // The MockAttributesFetcher will return this attestation type in the issuable documents.
        let mock_disclosed_type = "attestation_type";
        let mock_disclosed_attrs = mock_disclosed_attrs(mock_disclosed_type.to_string());

        let query_params = &result_handler
            .disclosure_result("usecase_id", &mock_disclosed_attrs)
            .await
            .unwrap();
        let credential_offer: CredentialOffer = serde_json::from_str(&query_params["credential_offer"]).unwrap();

        let code = credential_offer.grants.as_ref().unwrap().authorization_code().unwrap();

        // The session handler should have inserted a new issuance session in the session store.
        let IssuanceData::Created(session) = issuance_sessions
            .get(&SessionToken::from(code.as_ref().to_string()))
            .await
            .unwrap()
            .unwrap()
            .data
        else {
            panic!("session absent or in unexpected state")
        };

        // The session should contain an issuable attestation with our earlier disclosed attestation type.
        let issuable = session.issuable_documents.as_ref().unwrap().as_ref().first().unwrap();
        assert_eq!(issuable.attestation_type(), mock_disclosed_type);
    }

    #[tokio::test]
    async fn no_attestations_error() {
        let issuance_sessions: Arc<MemorySessionStore<IssuanceData>> =
            Arc::new(MemorySessionStore::new(SessionStoreTimeouts::default()));

        let result_handler = IssuanceResultHandler {
            attributes_fetcher: MockAttributesFetcher(vec![]),
            issuance_sessions: Arc::clone(&issuance_sessions),
            credential_issuer: "https://example.com".parse().unwrap(),
        };

        assert!(matches!(
            result_handler
                .disclosure_result("usecase_id", &mock_disclosed_attrs("vct".to_string()))
                .await,
            Err(DisclosureResultHandlerError::WalletError(
                PostAuthResponseError::NoIssuableAttestations
            ))
        ));
    }
}
