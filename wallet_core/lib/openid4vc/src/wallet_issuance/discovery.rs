use crypto::trust_anchor::BorrowingTrustAnchor;
use http_utils::reqwest::HttpJsonClient;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::credential_offer::CredentialOfferContainer;
use crate::credential_offer::Grants;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::metadata::well_known;
use crate::metadata::well_known::WellKnownPath;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::wallet_issuance::IssuanceDiscovery;
use crate::wallet_issuance::WalletIssuanceError;
use crate::wallet_issuance::authorization::HttpAuthorizationSession;
use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
use crate::wallet_issuance::issuance_session::HttpVcMessageClient;

pub struct HttpIssuanceDiscovery {
    http_client: HttpJsonClient,
}

impl HttpIssuanceDiscovery {
    pub fn new(http_client: HttpJsonClient) -> Self {
        Self { http_client }
    }
}

impl IssuanceDiscovery for HttpIssuanceDiscovery {
    type Authorization = HttpAuthorizationSession;
    type Issuance = HttpIssuanceSession;

    async fn start_authorization_code_flow(
        &self,
        credential_issuer: &IssuerIdentifier,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError> {
        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(credential_issuer).await?;

        let session = HttpAuthorizationSession::create(
            self.http_client.clone(),
            issuer_metadata,
            oauth_metadata,
            client_id,
            redirect_uri,
        )
        .await?;
        Ok(session)
    }

    async fn start_pre_authorized_code_flow(
        &self,
        redirect_uri: &Url,
        client_id: String,
        trust_anchors: &[BorrowingTrustAnchor],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let credential_offer = self.process_credential_offer(redirect_uri).await?;

        let pre_authorized_code = match credential_offer.flow {
            CredentialOfferFlow::AuthorizationCode { .. } => {
                // TODO (PVW-5832): Remove when implementing CredentialOffer Authorization Code flow.
                return Err(WalletIssuanceError::MissingCredentialOfferPreAuthorizedCode);
            }
            CredentialOfferFlow::PreAuthorizedCode { pre_authorized_code } => pre_authorized_code,
        };

        // TODO (PVW-5528): Use the authorization server from the Credential Offer, if provided.
        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(&credential_offer.credential_issuer).await?;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: None,
            client_id: Some(client_id),
            redirect_uri: None,
        };

        let message_client = HttpVcMessageClient::new(self.http_client.clone());

        HttpIssuanceSession::create(
            message_client,
            issuer_metadata,
            oauth_metadata,
            token_request,
            trust_anchors,
        )
        .await
    }
}

#[derive(Debug)]
struct NormalizedCredentialOffer {
    credential_issuer: IssuerIdentifier,
    // TODO (PVW-5856): Use this field when all issuance starts with a CredentialOffer.
    #[expect(dead_code)]
    credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,
    // TODO (PVW-5528): Use this field when considering which authorization server to choose.
    #[expect(dead_code)]
    authorization_server: Option<IssuerIdentifier>,
    flow: CredentialOfferFlow,
}

#[derive(Debug)]
enum CredentialOfferFlow {
    // TODO (PVW-5832): Use CredentialOffer for Authorization Code flow.
    #[expect(dead_code)]
    AuthorizationCode {
        issuer_state: Option<String>,
    },
    PreAuthorizedCode {
        pre_authorized_code: AuthorizationCode,
    },
}

impl HttpIssuanceDiscovery {
    async fn process_credential_offer(
        &self,
        redirect_uri: &Url,
    ) -> Result<NormalizedCredentialOffer, WalletIssuanceError> {
        let query = redirect_uri
            .query()
            .ok_or(WalletIssuanceError::MissingCredentialOfferQuery)?;

        let offer_container = serde_urlencoded::from_str::<CredentialOfferContainer>(query)
            .map_err(WalletIssuanceError::CredentialOfferDeserialization)?;

        let offer = match offer_container {
            CredentialOfferContainer::Offer { credential_offer } => *credential_offer,
            CredentialOfferContainer::Uri { credential_offer_uri } => self
                .http_client
                .get(credential_offer_uri.into_inner())
                .await
                .map_err(WalletIssuanceError::CredentialOfferHttp)?,
        };

        let grants = offer.grants.ok_or(WalletIssuanceError::MissingCredentialOfferGrants)?;

        let (authorization_server, flow) = match grants {
            Grants::Both {
                pre_authorized_code, ..
            }
            | Grants::PreAuthorizedCode { pre_authorized_code } => {
                if pre_authorized_code.tx_code.is_some() {
                    return Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported);
                }

                let flow = CredentialOfferFlow::PreAuthorizedCode {
                    pre_authorized_code: pre_authorized_code.pre_authorized_code,
                };

                (pre_authorized_code.authorization_server, flow)
            }
            Grants::AuthorizationCode { authorization_code } => {
                let flow = CredentialOfferFlow::AuthorizationCode {
                    issuer_state: authorization_code.issuer_state,
                };

                (authorization_code.authorization_server, flow)
            }
        };

        let normalized = NormalizedCredentialOffer {
            credential_issuer: offer.credential_issuer,
            credential_configuration_ids: offer.credential_configuration_ids,
            authorization_server,
            flow,
        };

        Ok(normalized)
    }

    async fn fetch_metadata(
        &self,
        credential_issuer: &IssuerIdentifier,
    ) -> Result<(IssuerMetadata, AuthorizationServerMetadata), WalletIssuanceError> {
        let issuer_metadata: IssuerMetadata =
            well_known::fetch_well_known(&self.http_client, credential_issuer, WellKnownPath::CredentialIssuer)
                .await
                .map_err(WalletIssuanceError::CredentialIssuerDiscovery)?;

        // Note: the spec allows multiple authorization servers, but we currently only support one.
        let auth_server = issuer_metadata.authorization_servers().into_first();

        let oauth_metadata: AuthorizationServerMetadata =
            well_known::fetch_well_known(&self.http_client, auth_server, WellKnownPath::OauthAuthorizationServer)
                .await
                .map_err(WalletIssuanceError::OauthDiscovery)?;

        Ok((issuer_metadata, oauth_metadata))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::num::NonZeroU8;

    use assert_matches::assert_matches;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::BorrowingTrustAnchor;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use serde_json::json;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use super::HttpIssuanceDiscovery;
    use super::IssuanceDiscovery;
    use crate::Format;
    use crate::credential_offer::CredentialOffer;
    use crate::credential_offer::CredentialOfferContainer;
    use crate::credential_offer::GrantAuthorizationCode;
    use crate::credential_offer::GrantPreAuthorizedCode;
    use crate::credential_offer::Grants;
    use crate::credential_offer::PreAuthTransactionCode;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::preview::CredentialPreviewResponse;
    use crate::token::CredentialPreview;
    use crate::token::CredentialPreviewContent;
    use crate::token::TokenResponse;
    use crate::token::TokenType;
    use crate::wallet_issuance::AuthorizationSession;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_httpmock_issuer(
        authorization_endpoint: Option<&str>,
    ) -> (MockServer, IssuerIdentifier, BorrowingTrustAnchor) {
        let server = MockServer::start_async().await;
        let issuer_identifier = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Create CA and issuer certificate for the credential preview.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_pid_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let trust_anchor = ca.to_borrowing_trust_anchor();

        // Create type metadata for the credential preview.
        let (_, _, type_metadata_documents) =
            TypeMetadataDocuments::from_single_example(TypeMetadata::example_with_claim_name(
                PID_ATTESTATION_TYPE,
                "family_name",
                JsonSchemaPropertyType::String,
                None,
            ));

        let credential_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let preview = CredentialPreview {
            content: CredentialPreviewContent {
                format: Format::MsoMdoc,
                batch_size: NonZeroU8::new(4).unwrap(),
                credential_payload,
                issuer_certificate: issuance_keypair.certificate().clone(),
            },
            type_metadata: type_metadata_documents,
        };

        let preview_response = CredentialPreviewResponse {
            credential_previews: vec_nonempty![preview],
        };

        let token_response = TokenResponse {
            access_token: "mock_access_token".to_string().into(),
            token_type: TokenType::DPoP,
            refresh_token: None,
            scope: None,
            expires_in: None,
            authorization_details: None,
        };

        // Construct issuer metadata JSON.
        let issuer_metadata_json = json!({
            "credential_issuer": issuer_identifier.to_string(),
            "credential_endpoint": server.url("/issuance/credential"),
            "batch_credential_endpoint": server.url("/issuance/batch_credential"),
            "nonce_endpoint": server.url("/issuance/nonce"),
            "credential_preview_endpoint": server.url("/issuance/credential_preview"),
            "credential_configurations_supported": {
                PID_ATTESTATION_TYPE: {
                    "format": "mso_mdoc",
                    "doctype": PID_ATTESTATION_TYPE,
                    "proof_types_supported": {
                        "jwt": { "proof_signing_alg_values_supported": ["ES256"] }
                    },
                }
            },
        });

        // Construct OAuth metadata JSON.
        let mut oauth_metadata_json = json!({
            "issuer": issuer_identifier.to_string(),
            "token_endpoint": server.url("/issuance/token"),
            "response_types_supported": ["code"],
            "subject_types_supported": [],
            "id_token_signing_alg_values_supported": [],
        });
        if let Some(auth_endpoint) = authorization_endpoint {
            oauth_metadata_json["authorization_endpoint"] = json!(auth_endpoint);
            oauth_metadata_json["pushed_authorization_request_endpoint"] = json!(server.url("/issuance/par"));
        }

        server
            .mock_async(|when, then| {
                when.method(GET).path("/.well-known/openid-credential-issuer");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(issuer_metadata_json);
            })
            .await;

        server
            .mock_async(|when, then| {
                when.method(GET).path("/.well-known/oauth-authorization-server");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(oauth_metadata_json);
            })
            .await;

        if authorization_endpoint.is_some() {
            server
                .mock_async(|when, then| {
                    when.method(POST).path("/issuance/par");
                    then.status(201)
                        .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                        .json_body(json!({
                            "request_uri": "urn:ietf:params:oauth:request_uri:mock-test-uri",
                            "expires_in": 60,
                        }));
                })
                .await;
        }

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .header("DPoP-Nonce", "mock_dpop_nonce")
                    .json_body(serde_json::to_value(token_response).unwrap());
            })
            .await;

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/credential_preview");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .header("DPoP-Nonce", "mock_dpop_nonce")
                    .json_body(serde_json::to_value(preview_response).unwrap());
            })
            .await;

        (server, issuer_identifier, trust_anchor)
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_offer_by_value() {
        let (_server, issuer_identifier, trust_anchor) = start_httpmock_issuer(None).await;

        // Construct a credential offer URL with a fake pre-authorized code.
        let offer_container = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            "fake_pre_auth_code".to_string().into(),
        ));
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let session = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[trust_anchor])
            .await
            .unwrap();

        assert_eq!(session.normalized_credential_preview().len(), 1);
        assert_eq!(
            session.normalized_credential_preview()[0]
                .content
                .credential_payload
                .attestation_type,
            PID_ATTESTATION_TYPE
        );
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_offer_by_reference() {
        let (server, issuer_identifier, trust_anchor) = start_httpmock_issuer(None).await;

        // Construct a credential offer URL with a fake pre-authorized code.
        let offer = CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            "fake_pre_auth_code".to_string().into(),
        );

        server
            .mock_async(|when, then| {
                when.method(GET).path("/credential_offer");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(serde_json::to_value(offer).unwrap());
            })
            .await;

        // Construct a credential offer URL that contains the Credential Offer URI.
        let offer_container = CredentialOfferContainer::new_uri(server.url("/credential_offer").parse().unwrap());
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let session = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[trust_anchor])
            .await
            .unwrap();

        assert_eq!(session.normalized_credential_preview().len(), 1);
        assert_eq!(
            session.normalized_credential_preview()[0]
                .content
                .credential_payload
                .attestation_type,
            PID_ATTESTATION_TYPE
        );
    }

    #[tokio::test]
    async fn authorization_code_flow() {
        let authorization_endpoint = "https://auth.example.com/authorize";
        let (_server, issuer_identifier, trust_anchor) = start_httpmock_issuer(Some(authorization_endpoint)).await;

        let redirect_uri: Url = "https://wallet.example.com/callback".parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        // Start the authorization code flow — fetches metadata and creates an auth session.
        let auth_session = discovery
            .start_authorization_code_flow(
                &issuer_identifier,
                MOCK_WALLET_CLIENT_ID.to_string(),
                redirect_uri.clone(),
            )
            .await
            .unwrap();

        // Verify the auth URL points to the expected authorization endpoint and carries PAR params.
        assert!(auth_session.auth_url().as_str().starts_with(authorization_endpoint));
        let auth_params: HashMap<String, String> = auth_session
            .auth_url()
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert!(auth_params.contains_key("request_uri"));
        assert!(!auth_params.contains_key("state"));

        // State is carried inside the PAR-stored request, not the auth URL; read it from the session.
        let state = auth_session.state().to_owned();

        // Simulate the authorization server redirecting back with a code and state.
        let mut received_redirect_uri = redirect_uri;
        received_redirect_uri.set_query(Some(&format!("code=fake_auth_code&state={state}")));

        // Complete the flow — exchanges the code for a token and fetches credential previews.
        let session = auth_session
            .start_issuance(&received_redirect_uri, &[trust_anchor])
            .await
            .unwrap();

        assert_eq!(session.normalized_credential_preview().len(), 1);
        assert_eq!(
            session.normalized_credential_preview()[0]
                .content
                .credential_payload
                .attestation_type,
            PID_ATTESTATION_TYPE
        );
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_missing_query() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_deserialization_error() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferDeserialization(_)));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_credential_offer_http_error() {
        let server = MockServer::start_async().await;

        // Construct a Credential Offer that contains an invalid URI.
        let offer_container = CredentialOfferContainer::new_uri(server.url("/does-not-exist").parse().unwrap());
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferHttp(_)));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_missing_grant() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct a Credential Offer URL WITHOUT any grants.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            grants: None,
        };
        let offer_container = CredentialOfferContainer::new_offer(credential_offer);
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferGrants));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_credential_offer_tx_code() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct a Pre-Authorized Code Credential Offer with a Transaction Code.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            grants: Some(Grants::PreAuthorizedCode {
                pre_authorized_code: GrantPreAuthorizedCode {
                    pre_authorized_code: "code".to_string().into(),
                    tx_code: Some(PreAuthTransactionCode::default()),
                    authorization_server: None,
                },
            }),
        };
        let offer_container = CredentialOfferContainer::new_offer(credential_offer);
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_authorization_code_grant() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct an Authorization Code Credential Offer.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            grants: Some(Grants::AuthorizationCode {
                authorization_code: GrantAuthorizationCode {
                    issuer_state: None,
                    authorization_server: None,
                },
            }),
        };
        let offer_container = CredentialOfferContainer::new_offer(credential_offer);
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::MissingCredentialOfferPreAuthorizedCode)
        );
    }
}
