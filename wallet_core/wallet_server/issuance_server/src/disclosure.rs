use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use indexmap::IndexMap;
use indexmap::IndexSet;

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
use openid4vc::server_state::SessionToken;
use openid4vc::token::TokenRequest;
use openid4vc::verifier::DisclosureResultHandler;
use wallet_common::reqwest;
use wallet_common::urls::BaseUrl;
use wallet_common::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DisclosureBasedAttributeError(#[from] anyhow::Error);

pub struct DisclosureBasedAttributeService<IS> {
    issuance_sessions: Arc<IS>,
}

impl<IS> DisclosureBasedAttributeService<IS> {
    pub fn new(issuance_sessions: Arc<IS>) -> Self {
        Self { issuance_sessions }
    }
}

impl<IS> AttributeService for DisclosureBasedAttributeService<IS>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
{
    type Error = DisclosureBasedAttributeError;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let issuance_data = self
            .issuance_sessions
            .get(&token_request.code().clone().into())
            .await
            .context("failed to get issuance session")?
            .ok_or(anyhow!(
                "issuance session not found: {0}",
                token_request.code().as_ref()
            ))?
            .data;

        let IssuanceData::Created(created) = issuance_data else {
            return Err(anyhow!("issuance session in unexpected state").into());
        };

        let issuable_documents = created
            .issuable_documents
            .ok_or(anyhow!("no attributes to be issued"))?;

        Ok(issuable_documents)
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        Ok(oidc::Config {
            issuer: issuer_url.clone(),
            authorization_endpoint: issuer_url.join_base_url("authorize").into_inner(),
            token_endpoint: issuer_url.join_base_url("token").into_inner(),
            userinfo_endpoint: None,
            jwks_uri: issuer_url.join_base_url("jwks").into_inner(),
            registration_endpoint: None,
            scopes_supported: None,
            response_types_supported: IndexSet::new(),
            response_modes_supported: None,
            grant_types_supported: None,
            acr_values_supported: None,
            subject_types_supported: IndexSet::new(),
            id_token_signing_alg_values_supported: IndexSet::new(),
            id_token_encryption_alg_values_supported: None,
            id_token_encryption_enc_values_supported: None,
            userinfo_signing_alg_values_supported: None,
            userinfo_encryption_alg_values_supported: None,
            userinfo_encryption_enc_values_supported: None,
            request_object_signing_alg_values_supported: None,
            request_object_encryption_alg_values_supported: None,
            request_object_encryption_enc_values_supported: None,
            token_endpoint_auth_methods_supported: None,
            token_endpoint_auth_signing_alg_values_supported: None,
            display_values_supported: None,
            claim_types_supported: None,
            claims_supported: None,
            service_documentation: None,
            claims_locales_supported: None,
            ui_locales_supported: None,
            claims_parameter_supported: false,
            request_parameter_supported: false,
            request_uri_parameter_supported: false,
            require_request_uri_registration: false,
            op_policy_uri: None,
            op_tos_uri: None,
            code_challenge_methods_supported: None,
        })
    }
}

#[trait_variant::make(Send)]
pub trait AttributesFetcher {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<VecNonEmpty<IssuableDocument>, Self::Error>;
}

pub struct HttpAttributesFetcher {
    pub urls: HashMap<String, BaseUrl>,
}

impl AttributesFetcher for HttpAttributesFetcher {
    type Error = DisclosureBasedAttributeError;

    async fn attributes(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let usecase_url = self
            .urls
            .get(usecase_id)
            .ok_or(anyhow!("unknown usecase: {usecase_id}"))?
            .as_ref()
            .clone();

        let to_issue = reqwest::default_reqwest_client_builder()
            .build()
            .context("failed to construct reqwest instance")?
            .post(usecase_url)
            .json(disclosed)
            .send()
            .await
            .context("failed to fetch attributes to be issued")?
            .json()
            .await
            .context("failed to deserialize attributes to be issued")?;

        Ok(to_issue)
    }
}

pub struct IssuanceResultHandler<IS, A> {
    pub attributes_fetcher: A,
    pub issuance_sessions: Arc<IS>,
    pub credential_issuer: BaseUrl,
}

impl<IS, A> DisclosureResultHandler for IssuanceResultHandler<IS, A>
where
    IS: SessionStore<IssuanceData> + Send + Sync + 'static,
    A: AttributesFetcher + Sync + 'static,
{
    type Error = DisclosureBasedAttributeError;

    async fn disclosure_result(
        &self,
        usecase_id: &str,
        disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
    ) -> Result<String, Self::Error> {
        let to_issue = self
            .attributes_fetcher
            .attributes(usecase_id, disclosed)
            .await
            .context("failed to fetch attributes")?;

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
            .context("failed to write issuance session")?;

        let credential_offer = CredentialOffer {
            credential_issuer: self.credential_issuer.clone(),
            credential_configuration_ids: vec![], // TODO
            grants: Some(Grants::PreAuthorizedCode {
                pre_authorized_code: GrantPreAuthorizedCode {
                    pre_authorized_code: token.as_ref().clone().into(),
                    tx_code: None,
                    authorization_server: None,
                },
            }),
        };

        let credential_offer = serde_urlencoded::to_string(CredentialOfferContainer { credential_offer })
            .context("failed URL-encode credential offer")?;

        Ok(credential_offer)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use indexmap::IndexMap;

    use mdoc::verifier::DocumentDisclosedAttributes;
    use openid4vc::issuable_document::IssuableDocument;
    use wallet_common::vec_at_least::VecNonEmpty;

    use super::AttributesFetcher;

    pub struct MockAttributesFetcher(pub VecNonEmpty<IssuableDocument>);

    impl AttributesFetcher for MockAttributesFetcher {
        type Error = Infallible;

        async fn attributes(
            &self,
            _usecase_id: &str,
            _disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
            Ok(self.0.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
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
    use wallet_common::vec_at_least::VecNonEmpty;

    use super::AttributesFetcher;
    use super::IssuanceResultHandler;

    pub struct TestAttributesFetcher;

    impl AttributesFetcher for TestAttributesFetcher {
        type Error = Infallible;

        async fn attributes(
            &self,
            _usecase_id: &str,
            disclosed: &IndexMap<String, DocumentDisclosedAttributes>,
        ) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
            // Insert the received attribute type into the issuable document so the caller can see we did our job
            let (attestation_type, _) = disclosed.first().unwrap();

            Ok(vec![IssuableDocument::try_new(
                attestation_type.clone(),
                IndexMap::from([(
                    "attr_name".to_string(),
                    Attribute::Single(AttributeValue::Text("attrvalue".to_string())),
                )]),
            )
            .unwrap()]
            .try_into()
            .unwrap())
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

        let mock_disclosed_attrs = IndexMap::from([(
            mock_disclosed_type.to_string(),
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
        )]);

        let credential_offer = result_handler
            .disclosure_result("usecase_id", &mock_disclosed_attrs)
            .await
            .unwrap();

        let CredentialOfferContainer { credential_offer } = serde_urlencoded::from_str(&credential_offer).unwrap();
        let code = credential_offer.grants.unwrap().authorization_code().unwrap();

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
}
