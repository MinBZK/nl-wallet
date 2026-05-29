use crypto::trust_anchor::TrustAnchors;
use http_utils::reqwest::HttpJsonClient;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use super::IssuanceDiscovery;
use super::IssuanceFlow;
use super::WalletIssuanceError;
use super::authorization::HttpAuthorizationSession;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
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

    async fn start_with_credential_offer(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError> {
        let credential_offer = self.process_credential_offer(offer_uri).await?;

        // TODO (PVW-5528): Use the authorization server from the Credential Offer, if provided.
        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(&credential_offer.credential_issuer).await?;

        let http_client = self.http_client.clone();

        let flow = match credential_offer.flow {
            CredentialOfferFlow::AuthorizationCode { issuer_state } => {
                let authorization_session = HttpAuthorizationSession::create(
                    http_client,
                    issuer_metadata,
                    oauth_metadata,
                    client_id,
                    redirect_uri,
                    issuer_state,
                )
                .await?;

                IssuanceFlow::AuthorizationCode { authorization_session }
            }
            CredentialOfferFlow::PreAuthorizedCode { pre_authorized_code } => {
                let token_request = TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
                    code_verifier: None,
                    client_id: Some(client_id),
                    redirect_uri: None,
                };

                let message_client = HttpVcMessageClient::new(http_client);

                let issuance_session = HttpIssuanceSession::create(
                    message_client,
                    issuer_metadata,
                    oauth_metadata,
                    token_request,
                    issuer_trust_anchors,
                )
                .await?;

                IssuanceFlow::PreAuthorizedCode { issuance_session }
            }
            CredentialOfferFlow::NoGrants => {
                // According to the OpenID4VCI 1.0 specification:
                // (source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-2.3)
                //
                // "If grants is not present or is empty, the Wallet MUST determine the Grant Types the Credential
                // Issuer's Authorization Server supports using the respective metadata. "
                //
                // Since a Pre-Authorized Code grant type without an actual code does not make any sense, we only check
                // for the Authorization Code grant type here and use that if the Authorization Server supports it.
                if !oauth_metadata
                    .grant_types_supported
                    .as_ref()
                    .map(|grant_types| grant_types.contains("authorization_code"))
                    // RFC 8414 says about the "grant_types" field:
                    // (source: https://datatracker.ietf.org/doc/html/rfc8414#section-2)
                    //
                    // If omitted, the default value is "["authorization_code", "implicit"]".
                    .unwrap_or(true)
                {
                    return Err(WalletIssuanceError::AuthorizationCodeNotSupported);
                }

                let authorization_session = HttpAuthorizationSession::create(
                    http_client,
                    issuer_metadata,
                    oauth_metadata,
                    client_id,
                    redirect_uri,
                    None,
                )
                .await?;

                IssuanceFlow::AuthorizationCode { authorization_session }
            }
        };

        Ok(flow)
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

#[derive(Debug, Clone)]
enum CredentialOfferFlow {
    AuthorizationCode { issuer_state: Option<String> },
    PreAuthorizedCode { pre_authorized_code: AuthorizationCode },
    NoGrants,
}

impl CredentialOfferFlow {
    fn from_grants(grants: Grants) -> Result<(Self, Option<IssuerIdentifier>), WalletIssuanceError> {
        let result = match (grants.authorization_code, grants.pre_authorized_code) {
            // According to the OpenID4VCI 1.0 specification:
            // (source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-2.3)
            // "When multiple grants are present, it is at the Wallet's discretion which one to use."
            //
            // Since there is no point in making the user go through an OAuth authorization flow when they have already
            // been pre-authorized, we always choose the pre-authorized code from the `CredentialOffer` if it is
            // available.
            (_, Some(pre_authorized_code)) => {
                if pre_authorized_code.tx_code.is_some() {
                    return Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported);
                }

                let flow = Self::PreAuthorizedCode {
                    pre_authorized_code: pre_authorized_code.pre_authorized_code,
                };

                (flow, pre_authorized_code.authorization_server)
            }
            (Some(authorization_code), None) => {
                let flow = Self::AuthorizationCode {
                    issuer_state: authorization_code.issuer_state,
                };

                (flow, authorization_code.authorization_server)
            }
            (None, None) => {
                // If the Credential Offer does not contain an Authorization Code or Pre-Authorized Code grant, but does
                // contain some other unknown grant type, we should not fall back to consulting the Authorization Server
                // metadata for supported grant types. As we do not support the grant type, this is an error case.
                if !grants.unknown.is_empty() {
                    return Err(WalletIssuanceError::CredentialOfferUnknownGrants(
                        grants.unknown.into_keys().collect(),
                    ));
                }

                (Self::NoGrants, None)
            }
        };

        Ok(result)
    }
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

        let (flow, authorization_server) = if let Some(grants) = offer.grants {
            CredentialOfferFlow::from_grants(grants)?
        } else {
            (CredentialOfferFlow::NoGrants, None)
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
    use std::assert_matches;
    use std::collections::HashMap;
    use std::num::NonZeroU8;
    use std::sync::LazyLock;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use itertools::Itertools;
    use rstest::rstest;
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
    use crate::wallet_issuance::IssuanceFlow;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;

    const DEFAULT_GRANT_TYPES_SUPPORTED: &[&str] = &[
        "authorization_code",
        "urn:ietf:params:oauth:grant-type:pre-authorized_code",
    ];
    static REDIRECT_URI: LazyLock<Url> = LazyLock::new(|| "https://wallet.example.com/callback".parse().unwrap());
    const AUTHORIZATION_ENDPOINT: &str = "https://auth.example.com/authorize";

    async fn httpmock_issuer_add_metadata(
        server: &MockServer,
        issuer_identifier: &IssuerIdentifier,
        grant_types_supported: Option<&[&str]>,
    ) {
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
            "authorization_endpoint": AUTHORIZATION_ENDPOINT,
            "token_endpoint": server.url("/issuance/token"),
            "response_types_supported": ["code"],
            "subject_types_supported": [],
            "id_token_signing_alg_values_supported": [],
            "pushed_authorization_request_endpoint": server.url("/issuance/par")
        });
        if let Some(grant_types_supported) = grant_types_supported {
            oauth_metadata_json["grant_types_supported"] = json!(grant_types_supported);
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
    }

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_httpmock_issuer(has_grant_types_supported: bool) -> (MockServer, IssuerIdentifier, TrustAnchors) {
        let server = MockServer::start_async().await;
        let issuer_identifier = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Create CA and issuer certificate for the credential preview.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_pid_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let trust_anchor = TrustAnchors::from(&ca);

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

        httpmock_issuer_add_metadata(
            &server,
            &issuer_identifier,
            has_grant_types_supported.then_some(DEFAULT_GRANT_TYPES_SUPPORTED),
        )
        .await;

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

    #[derive(Debug)]
    enum IssuanceDiscoveryScenario {
        AuthorizationCode,
        PreAuthorizedCode,
        NoGrants { has_grant_types_supported: bool },
        EmptyGrants { has_grant_types_supported: bool },
    }

    #[rstest]
    #[case::authorization_code(IssuanceDiscoveryScenario::AuthorizationCode)]
    #[case::pre_authorized_code(IssuanceDiscoveryScenario::PreAuthorizedCode)]
    #[case::no_grants(IssuanceDiscoveryScenario::NoGrants { has_grant_types_supported: true })]
    #[case::no_grants_no_grant_types(IssuanceDiscoveryScenario::NoGrants { has_grant_types_supported: false })]
    #[case::empty_grants(IssuanceDiscoveryScenario::EmptyGrants { has_grant_types_supported: true })]
    #[case::empty_grants_no_grant_types(IssuanceDiscoveryScenario::EmptyGrants { has_grant_types_supported: false })]
    #[tokio::test]
    async fn http_issuance_discovery_start_with_credential_offer(
        #[case] scenario: IssuanceDiscoveryScenario,
        #[values(false, true)] is_by_reference: bool,
    ) {
        // Start a mock issuance server, which may or may not have a "grant_types_supported" field.
        let has_grant_types_supported = match &scenario {
            IssuanceDiscoveryScenario::AuthorizationCode | IssuanceDiscoveryScenario::PreAuthorizedCode => true,
            IssuanceDiscoveryScenario::NoGrants {
                has_grant_types_supported,
            }
            | IssuanceDiscoveryScenario::EmptyGrants {
                has_grant_types_supported,
            } => *has_grant_types_supported,
        };

        let (server, issuer_identifier, trust_anchor) = start_httpmock_issuer(has_grant_types_supported).await;

        // Construct a Credential Offer based on the scenario.
        let grants = match scenario {
            IssuanceDiscoveryScenario::AuthorizationCode => Some(Grants::new_authorization(None)),
            IssuanceDiscoveryScenario::PreAuthorizedCode => {
                Some(Grants::new_pre_authorized("fake_pre_auth_code".to_string().into()))
            }
            IssuanceDiscoveryScenario::NoGrants { .. } => None,
            IssuanceDiscoveryScenario::EmptyGrants { .. } => Some(Grants::default()),
        };
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier,
            credential_configuration_ids: vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            grants,
        };

        // If the Credential Offer is by reference, have the mock issuance server serve it. Construct the Credential
        // Offer URL based on this.
        let offer_container = if is_by_reference {
            server
                .mock_async(|when, then| {
                    when.method(GET).path("/credential_offer");

                    then.status(200)
                        .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                        .json_body(serde_json::to_value(&credential_offer).unwrap());
                })
                .await;

            CredentialOfferContainer::new_uri(server.url("/credential_offer").parse().unwrap())
        } else {
            CredentialOfferContainer::new_offer(credential_offer)
        };
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url = format!("openid-credential-offer://?{query}").parse::<Url>().unwrap();

        // Start issuance based on this Credential Offer URL.
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let flow = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
            )
            .await
            .expect("starting issuance should succeed");

        let issuance_session = match (scenario, flow) {
            (
                IssuanceDiscoveryScenario::AuthorizationCode,
                IssuanceFlow::AuthorizationCode {
                    authorization_session: auth_session,
                },
            )
            | (
                IssuanceDiscoveryScenario::NoGrants { .. },
                IssuanceFlow::AuthorizationCode {
                    authorization_session: auth_session,
                },
            )
            | (
                IssuanceDiscoveryScenario::EmptyGrants { .. },
                IssuanceFlow::AuthorizationCode {
                    authorization_session: auth_session,
                },
            ) => {
                // Verify the auth URL points to the expected authorization endpoint and carries PAR params.
                assert!(auth_session.auth_url().as_str().starts_with(AUTHORIZATION_ENDPOINT));
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
                let mut received_redirect_uri = REDIRECT_URI.clone();
                received_redirect_uri.set_query(Some(&format!("code=fake_auth_code&state={state}")));

                // Complete the flow — exchanges the code for a token and fetches credential previews.
                auth_session
                    .start_issuance(&received_redirect_uri, &trust_anchor)
                    .await
                    .unwrap()
            }
            (IssuanceDiscoveryScenario::PreAuthorizedCode, IssuanceFlow::PreAuthorizedCode { issuance_session }) => {
                // In case of the pre-authorized flow, we directly have an issuance session.
                issuance_session
            }
            _ => {
                panic!("unexpected issuance flow type received");
            }
        };

        // Check that the issuance session contains the expected credential preview.
        assert_eq!(issuance_session.normalized_credential_preview().len(), 1);
        assert_eq!(
            issuance_session.normalized_credential_preview()[0]
                .content
                .credential_payload
                .attestation_type,
            PID_ATTESTATION_TYPE
        );
    }

    #[tokio::test]
    async fn start_with_credential_offer_missing_query() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery));
    }

    #[tokio::test]
    async fn start_with_credential_offer_deserialization_error() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferDeserialization(_)));
    }

    #[tokio::test]
    async fn start_with_credential_offer_credential_offer_http_error() {
        let server = MockServer::start_async().await;

        // Construct a Credential Offer that contains an invalid URI.
        let offer_container = CredentialOfferContainer::new_uri(server.url("/does-not-exist").parse().unwrap());
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferHttp(_)));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_unknown_grants() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct a Credential Offer URL with only unknown grant types.
        let credential_offer = json!({
            "credential_issuer": credential_issuer.as_ref(),
            "credential_configuration_ids": [PID_ATTESTATION_TYPE],
            "grants": {
                "foo": {
                    "key": "value"
                },
                "bar": {
                    "something": 123
                }
            }
        });
        let mut offer_url = Url::parse("openid-credential-offer://").unwrap();
        offer_url
            .query_pairs_mut()
            .append_pair("credential_offer", &serde_json::to_string(&credential_offer).unwrap());

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialOfferUnknownGrants(grant_types))
                if grant_types.iter().sorted().eq(["bar", "foo"])
        );
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_credential_offer_authorization_code_not_supported() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Have the OAuth Authorization Server metadata not include "authorization_code" as a supported grant type.
        httpmock_issuer_add_metadata(&server, &credential_issuer, Some(&["implicit"])).await;

        // Construct a Credential Offer that contains no grants.
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
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::AuthorizationCodeNotSupported));
    }

    #[tokio::test]
    async fn pre_authorized_code_flow_credential_offer_tx_code() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct a Pre-Authorized Code Credential Offer with a Transaction Code.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            grants: Some(Grants {
                pre_authorized_code: Some(GrantPreAuthorizedCode {
                    pre_authorized_code: "code".to_string().into(),
                    tx_code: Some(PreAuthTransactionCode::default()),
                    authorization_server: None,
                }),
                ..Grants::default()
            }),
        };
        let offer_container = CredentialOfferContainer::new_offer(credential_offer);
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start_with_credential_offer(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported));
    }
}
