use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::Serialize;

use mdoc::verifier::DocumentDisclosedAttributes;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::Created;
use openid4vc::issuer::IssuanceData;
use openid4vc::oidc;
use openid4vc::server_state::SessionState;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::server_state::SessionToken;
use openid4vc::token::TokenRequest;
use openid4vc::verifier::DisclosureResultHandler;
use openid4vc::verifier::DisclosureResultHandlerError;
use openid4vc::verifier::PostAuthResponseError;
use wallet_common::reqwest::default_reqwest_client_builder;
use wallet_common::urls::BaseUrl;
use wallet_common::vec_at_least::VecNonEmpty;

pub struct DisclosureBasedAttributeService<IS> {
    issuance_sessions: Arc<IS>,
}

impl<IS> DisclosureBasedAttributeService<IS> {
    pub fn new(issuance_sessions: Arc<IS>) -> Self {
        Self { issuance_sessions }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeServiceError {
    #[error("failed to get issuance session: {0}")]
    GetIssuanceSession(#[from] SessionStoreError),
    #[error("issuance session not found: {0}")]
    MissingIssuanceSession(SessionToken),
    #[error("issuance session in unexpected state")]
    IsuanceSessionUnexpectedState,
    #[error("no attributes to be issued")]
    NoIssuableDocuments,
}

impl<IS> AttributeService for DisclosureBasedAttributeService<IS>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
{
    type Error = AttributeServiceError;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let session_token = token_request.code().clone().into();
        let issuance_data = self
            .issuance_sessions
            .get(&session_token)
            .await?
            .ok_or_else(|| AttributeServiceError::MissingIssuanceSession(session_token.clone()))?
            .data;

        let IssuanceData::Created(created) = issuance_data else {
            return Err(AttributeServiceError::IsuanceSessionUnexpectedState);
        };

        let issuable_documents = created
            .issuable_documents
            .ok_or_else(|| AttributeServiceError::NoIssuableDocuments)?;

        Ok(issuable_documents)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        // TODO (PVW-4257): we don't use the `authorize` and `jwks` endpoint here, but we need to specify them
        // because they are mandatory in an OIDC Provider Metadata document (see
        // <https://openid.net/specs/openid-connect-discovery-1_0.html>).
        // However, OpenID4VCI says that this should return not an OIDC Provider Metadata document but an OAuth
        // Authorization Metadata document instead, see <https://www.rfc-editor.org/rfc/rfc8414.html>, which to
        // a large extent has the same fields but `authorize` and `jwks` are optional there.
        Ok(oidc::Config::new(
            issuer_url.clone(),
            issuer_url.join("authorize"),
            issuer_url.join("token"),
            issuer_url.join("jwks"),
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AttributesFetcherError {
    #[error("unknown usecase: {0}")]
    UnknownUsecase(String),
    #[error("failed to fetch attributes to be issued: {0}")]
    AttestationsFetching(#[from] reqwest::Error),
    #[error("failed to deserialize attributes to be issued: {0}")]
    AttestationsDeserializing(#[from] serde_json::Error),
}

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
    ) -> Result<impl Serialize + Clone + 'static, DisclosureResultHandlerError> {
        let to_issue = self
            .attributes_fetcher
            .attributes(usecase_id, disclosed)
            .await
            .map_err(|err| DisclosureResultHandlerError::Other(err.into()))?;

        let to_issue: VecNonEmpty<_> = to_issue
            .try_into()
            .map_err(|_| DisclosureResultHandlerError::WalletError(PostAuthResponseError::NoIssuableAttestations))?;

        let credential_configuration_ids = to_issue
            .iter()
            .map(|attestation| attestation.attestation_type().to_string())
            .collect();

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

        let credential_offer = CredentialOffer {
            credential_issuer: self.credential_issuer.clone(),
            credential_configuration_ids,
            grants: Some(Grants::PreAuthorizedCode {
                pre_authorized_code: GrantPreAuthorizedCode {
                    pre_authorized_code: token.as_ref().clone().into(),
                    tx_code: None,
                    authorization_server: None,
                },
            }),
        };

        Ok(CredentialOfferContainer { credential_offer })
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
    use openid4vc::credential::CredentialOfferContainer;
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
            // Insert the received attribute type into the issuable document so the caller can see we did our job
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

        // IssuanceResultsHandler always returns a CredentialOfferContainer.
        let credential_offer: &dyn std::any::Any = &result_handler
            .disclosure_result("usecase_id", &mock_disclosed_attrs)
            .await
            .unwrap();
        let CredentialOfferContainer { credential_offer } = credential_offer.downcast_ref().unwrap();

        let code = credential_offer.grants.as_ref().unwrap().authorization_code().unwrap();

        // The session handler should have inserted a new issuance session in the session store.
        let IssuanceData::Created(session) = issuance_sessions
            .get(&SessionToken::from(code.as_ref().to_string()))
            .await
            .unwrap()
            .unwrap()
            .data
        else {
            panic!("session in unexpected state")
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
