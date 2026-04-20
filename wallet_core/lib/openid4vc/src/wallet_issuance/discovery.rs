use rustls_pki_types::TrustAnchor;
use url::Url;

use http_utils::reqwest::HttpJsonClient;

use crate::credential::CredentialOfferContainer;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::metadata::well_known;
use crate::metadata::well_known::WellKnownPath;
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

        let session = HttpAuthorizationSession::try_new(
            self.http_client.clone(),
            issuer_metadata,
            oauth_metadata,
            client_id,
            redirect_uri,
        )?;
        Ok(session)
    }

    async fn start_pre_authorized_code_flow(
        &self,
        redirect_uri: &Url,
        client_id: String,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let query = redirect_uri
            .query()
            .ok_or(WalletIssuanceError::MissingCredentialOfferQuery)?;

        let CredentialOfferContainer { credential_offer } =
            serde_urlencoded::from_str(query).map_err(WalletIssuanceError::CredentialOfferDeserialization)?;

        let pre_authorized_code = credential_offer
            .pre_authorized_code()
            .cloned()
            .ok_or(WalletIssuanceError::MissingPreAuthorizedCodeGrant)?;

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

impl HttpIssuanceDiscovery {
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

    use http::header;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use indexmap::IndexMap;
    use rustls_pki_types::TrustAnchor;
    use serde_json::json;
    use url::Url;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use crypto::server_keys::generate::Ca;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use http_utils::reqwest::default_reqwest_client_builder;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use crate::Format;
    use crate::credential::CredentialOffer;
    use crate::credential::CredentialOfferContainer;
    use crate::credential::GrantPreAuthorizedCode;
    use crate::credential::Grants;
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

    use super::HttpIssuanceDiscovery;
    use super::IssuanceDiscovery;

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_wiremock_issuer(
        authorization_endpoint: Option<&str>,
    ) -> (MockServer, IssuerIdentifier, TrustAnchor<'static>) {
        let server = MockServer::start_async().await;
        let issuer_identifier = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Create CA and issuer certificate for the credential preview.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_pid_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let trust_anchor = ca.to_trust_anchor().to_owned();

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
                copies_per_format: IndexMap::from([(Format::MsoMdoc, NonZeroU8::new(4).unwrap())]),
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
    async fn pre_authorized_code_flow() {
        let (_server, issuer_identifier, trust_anchor) = start_wiremock_issuer(None).await;

        // Construct a credential offer URL with a fake pre-authorized code.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier,
            credential_configuration_ids: vec![PID_ATTESTATION_TYPE.to_string()],
            grants: Some(Grants::PreAuthorizedCode {
                pre_authorized_code: GrantPreAuthorizedCode::new("fake_pre_auth_code".to_string().into()),
            }),
        };
        let container = CredentialOfferContainer { credential_offer };
        let query = serde_urlencoded::to_string(&container).unwrap();
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
        let (_server, issuer_identifier, trust_anchor) = start_wiremock_issuer(Some(authorization_endpoint)).await;

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

        // Verify the auth URL points to the expected authorization endpoint.
        assert!(auth_session.auth_url().as_str().starts_with(authorization_endpoint));

        // Extract the state from the auth URL to simulate the redirect.
        let auth_params: HashMap<String, String> = auth_session
            .auth_url()
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let state = auth_params.get("state").unwrap();

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
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert!(matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery)));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_deserialization_error() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert!(matches!(
            result,
            Err(WalletIssuanceError::CredentialOfferDeserialization(_))
        ));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_missing_grant() {
        // Construct a credential offer URL WITHOUT any grants.
        let credential_offer = CredentialOffer {
            credential_issuer: "https://example.com".parse().unwrap(),
            credential_configuration_ids: vec![PID_ATTESTATION_TYPE.to_string()],
            grants: None,
        };
        let container = CredentialOfferContainer { credential_offer };
        let query = serde_urlencoded::to_string(&container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[])
            .await;

        assert!(matches!(
            result,
            Err(WalletIssuanceError::MissingPreAuthorizedCodeGrant)
        ));
    }
}
