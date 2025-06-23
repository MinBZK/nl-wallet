use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;
use itertools::Itertools;

use attestation_data::disclosure::DocumentDisclosedAttributes;
use attestation_data::issuable_document::IssuableDocument;
use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::verifier::DisclosureResultHandler;
use openid4vc::verifier::DisclosureResultHandlerError;
use openid4vc::verifier::ToPostAuthResponseErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum AttributesFetcherError {
    #[error("unknown usecase: {0}")]
    UnknownUsecase(String),
    #[error("failed to fetch attributes to be issued: {0}")]
    AttestationsFetching(#[from] reqwest::Error),
}

/// Represents types that can take disclosed attributes and respond with attestations to be issued.
#[trait_variant::make(Send)]
pub trait AttributesFetcher {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<Vec<IssuableDocument>, Self::Error>;
}

pub struct HttpAttributesFetcher {
    urls: HashMap<String, PinnedReqwestClient>,
}

impl HttpAttributesFetcher {
    pub fn try_new(urls: HashMap<String, TlsPinningConfig>) -> Result<Self, reqwest::Error> {
        let urls = urls
            .into_iter()
            .map(|(id, config)| Ok((id, config.try_into_json_client()?)))
            .try_collect()?;

        Ok(Self { urls })
    }
}

impl AttributesFetcher for HttpAttributesFetcher {
    type Error = AttributesFetcherError;

    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<Vec<IssuableDocument>, Self::Error> {
        let http_client = self
            .urls
            .get(usecase_id)
            .ok_or_else(|| AttributesFetcherError::UnknownUsecase(usecase_id.to_string()))?;

        let to_issue = http_client
            .send_custom_post(ReqwestClientUrl::Base, |request| request.json(disclosed))
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(to_issue)
    }
}

/// Receives disclosed attributes, exchanges those for attestations to be issued, and creates a new issuance session
/// by implementing [`DisclosureResultHandler`].
pub struct IssuanceResultHandler<AF, AS, K, S, W> {
    pub attributes_fetcher: AF,
    pub issuer: Arc<Issuer<AS, K, S, W>>,
    pub credential_issuer: BaseUrl,
}

#[derive(Debug, thiserror::Error)]
pub enum IssuanceResultHandlerError {
    #[error("failed to fetch attributes: {0}")]
    AttributesFetching(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("no attestations to issue")]
    NoIssuableAttestations,
    #[error("failed to create session: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("URL encoding failed: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
    #[error("URL decoding failed: {0}")]
    UrlDecoding(#[from] serde_urlencoded::de::Error),
}

impl ToPostAuthResponseErrorCode for IssuanceResultHandlerError {
    fn to_error_code(&self) -> PostAuthResponseErrorCode {
        match self {
            IssuanceResultHandlerError::NoIssuableAttestations => PostAuthResponseErrorCode::NoIssuableAttestations,
            _ => PostAuthResponseErrorCode::ServerError,
        }
    }
}

#[async_trait]
impl<AF, AS, K, S, W> DisclosureResultHandler for IssuanceResultHandler<AF, AS, K, S, W>
where
    AF: AttributesFetcher + Sync,
    AS: AttributeService + Sync,
    S: SessionStore<IssuanceData> + Sync,
    K: Send + Sync,
    W: Send + Sync,
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
            .map_err(|e| DisclosureResultHandlerError::new(IssuanceResultHandlerError::AttributesFetching(e.into())))?;

        // Return a specific error code if there are no attestations to be issued so the wallet
        // can distinguish this case from other (error) cases.
        let to_issue: VecNonEmpty<_> = to_issue
            .try_into()
            .map_err(|_| DisclosureResultHandlerError::new(IssuanceResultHandlerError::NoIssuableAttestations))?;

        let credential_configuration_ids = to_issue
            .iter()
            .map(|attestation| attestation.attestation_type().to_string())
            .collect();

        // Start a new issuance session.
        let token = self
            .issuer
            .new_session(to_issue)
            .await
            .map_err(|err| DisclosureResultHandlerError::new(IssuanceResultHandlerError::SessionStore(err)))?;

        let credential_offer = CredentialOfferContainer {
            credential_offer: CredentialOffer {
                credential_issuer: self.credential_issuer.clone(),
                credential_configuration_ids,
                grants: Some(Grants::PreAuthorizedCode {
                    pre_authorized_code: GrantPreAuthorizedCode::new(token.into()),
                }),
            },
        };

        // If `serde_urlencoded` would have something like `serde_json::Value` or `ciborium::Value`,
        // then this would be a lot less awkward.
        let query_params = serde_urlencoded::from_str(
            &serde_urlencoded::to_string(credential_offer)
                .map_err(|err| DisclosureResultHandlerError::new(IssuanceResultHandlerError::UrlEncoding(err)))?,
        )
        .map_err(|err| DisclosureResultHandlerError::new(IssuanceResultHandlerError::UrlDecoding(err)))?;

        Ok(query_params)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::sync::Arc;

    use chrono::Utc;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::disclosure::DisclosedAttributes;
    use attestation_data::disclosure::DocumentDisclosedAttributes;
    use attestation_data::disclosure::ValidityInfo;
    use attestation_data::issuable_document::IssuableDocument;
    use openid4vc::credential::CredentialOffer;
    use openid4vc::issuer::AttestationTypeConfig;
    use openid4vc::issuer::IssuanceData;
    use openid4vc::issuer::Issuer;
    use openid4vc::issuer::TrivialAttributeService;
    use openid4vc::issuer::WteConfig;
    use openid4vc::server_state::MemorySessionStore;
    use openid4vc::server_state::MemoryWteTracker;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionStoreTimeouts;
    use openid4vc::server_state::SessionToken;
    use openid4vc::verifier::DisclosureResultHandler;
    use openid4vc::PostAuthResponseErrorCode;

    use super::AttributesFetcher;
    use super::IssuanceResultHandler;

    pub struct TestAttributesFetcher;

    fn mock_disclosed_attrs(attestation_type: String) -> IndexMap<String, DocumentDisclosedAttributes> {
        IndexMap::from([(
            attestation_type,
            DocumentDisclosedAttributes {
                attributes: DisclosedAttributes::MsoMdoc(IndexMap::new()),
                issuer_uri: "https://example.com".parse().unwrap(),
                ca: "ca".to_string(),
                validity_info: ValidityInfo {
                    signed: Utc::now(),
                    valid_from: Utc::now(),
                    valid_until: Utc::now(),
                },
            },
        )])
    }

    impl AttributesFetcher for TestAttributesFetcher {
        type Error = Infallible;

        async fn attributes(
            &self,
            _usecase_id: &str,
            disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<Vec<IssuableDocument>, Self::Error> {
            // Insert the received attribute type into the issuable document to demonstrate that the
            // issued attributes can depend on the disclosed attributes.
            let (attestation_type, _) = disclosed.first().unwrap();

            Ok(vec![IssuableDocument::try_new(
                attestation_type.clone(),
                IndexMap::from([(
                    "attr_name".to_string(),
                    Attribute::Single(AttributeValue::Text("attrvalue".to_string())),
                )])
                .into(),
            )
            .unwrap()])
        }
    }

    pub struct MockAttributesFetcher(pub Vec<IssuableDocument>);

    impl AttributesFetcher for MockAttributesFetcher {
        type Error = Infallible;

        async fn attributes(
            &self,
            _usecase_id: &str,
            _disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<Vec<IssuableDocument>, Self::Error> {
            Ok(self.0.clone())
        }
    }

    type MockIssuer = Issuer<TrivialAttributeService, SigningKey, MemorySessionStore<IssuanceData>, MemoryWteTracker>;

    fn mock_issuer(sessions: Arc<MemorySessionStore<IssuanceData>>) -> MockIssuer {
        Issuer::new(
            sessions,
            TrivialAttributeService,
            HashMap::<std::string::String, AttestationTypeConfig<SigningKey>>::new().into(),
            &"https://example.com".parse().unwrap(),
            vec![],
            None::<WteConfig<MemoryWteTracker>>,
        )
    }

    #[tokio::test]
    async fn it_works() {
        let sessions = Arc::new(MemorySessionStore::new(SessionStoreTimeouts::default()));

        let result_handler = IssuanceResultHandler {
            attributes_fetcher: TestAttributesFetcher,
            issuer: Arc::new(mock_issuer(Arc::clone(&sessions))),
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
        let IssuanceData::Created(session) = sessions
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
        let result_handler = IssuanceResultHandler {
            attributes_fetcher: MockAttributesFetcher(vec![]),
            issuer: Arc::new(mock_issuer(Arc::new(MemorySessionStore::new(
                SessionStoreTimeouts::default(),
            )))),
            credential_issuer: "https://example.com".parse().unwrap(),
        };

        let err = result_handler
            .disclosure_result("usecase_id", &mock_disclosed_attrs("vct".to_string()))
            .await
            .unwrap_err();

        assert!(matches!(
            err.as_ref().to_error_code(),
            PostAuthResponseErrorCode::NoIssuableAttestations,
        ));
    }
}
