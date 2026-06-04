use std::collections::HashSet;

use crypto::trust_anchor::TrustAnchors;
use http_utils::reqwest::HttpJsonClient;
use itertools::Either;
use itertools::Itertools;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use super::AuthorizationSession;
use super::IssuanceDiscovery;
use super::IssuanceFlow;
use super::WalletIssuanceError;
use super::authorization::HttpAuthorizationSession;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
use crate::credential_offer::CredentialOffer;
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

    async fn start(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError> {
        let (credential_configuration_ids, flow, issuer_metadata, oauth_metadata) =
            self.resolve_credential_offer_flow(offer_uri).await?;

        let issuance_flow = match flow {
            CredentialOfferFlow::AuthorizationCode { issuer_state } => {
                let authorization_session = HttpAuthorizationSession::create(
                    self.http_client.clone(),
                    credential_configuration_ids,
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
                let issuance_session = self
                    .create_issuance_session(
                        pre_authorized_code,
                        credential_configuration_ids,
                        issuer_metadata,
                        oauth_metadata,
                        client_id,
                        issuer_trust_anchors,
                    )
                    .await?;

                IssuanceFlow::PreAuthorizedCode { issuance_session }
            }
        };

        Ok(issuance_flow)
    }

    async fn start_authorization_code_flow(
        &self,
        offer_uri: &Url,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError> {
        let (credential_configuration_ids, flow, issuer_metadata, oauth_metadata) =
            self.resolve_credential_offer_flow(offer_uri).await?;

        let CredentialOfferFlow::AuthorizationCode { issuer_state } = flow else {
            return Err(WalletIssuanceError::CredentialOfferNoAuthorizationCode);
        };

        HttpAuthorizationSession::create(
            self.http_client.clone(),
            credential_configuration_ids,
            issuer_metadata,
            oauth_metadata,
            client_id,
            redirect_uri,
            issuer_state,
        )
        .await
    }

    async fn start_pre_authorized_code_flow(
        &self,
        offer_uri: &Url,
        client_id: String,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let (credential_configuration_ids, flow, issuer_metadata, oauth_metadata) =
            self.resolve_credential_offer_flow(offer_uri).await?;

        let CredentialOfferFlow::PreAuthorizedCode { pre_authorized_code } = flow else {
            return Err(WalletIssuanceError::CredentialOfferNoPreAuthorizedCode);
        };

        self.create_issuance_session(
            pre_authorized_code,
            credential_configuration_ids,
            issuer_metadata,
            oauth_metadata,
            client_id,
            issuer_trust_anchors,
        )
        .await
    }

    fn restore_authorization_session(
        &self,
        data: <Self::Authorization as AuthorizationSession>::Persisted,
    ) -> Self::Authorization {
        HttpAuthorizationSession::restore(self.http_client.clone(), data)
    }
}

#[derive(Debug)]
struct NormalizedCredentialOffer {
    credential_issuer: IssuerIdentifier,
    credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,
    // TODO (PVW-5528): Use this field when considering which authorization server to choose.
    #[expect(dead_code)]
    authorization_server: Option<IssuerIdentifier>,
    grant: CredentialOfferGrant,
}

#[derive(Debug)]
enum CredentialOfferGrant {
    GrantWithFlow { flow: CredentialOfferFlow },
    NoKnownGrant,
}

#[derive(Debug)]
enum CredentialOfferFlow {
    AuthorizationCode { issuer_state: Option<String> },
    PreAuthorizedCode { pre_authorized_code: AuthorizationCode },
}

impl NormalizedCredentialOffer {
    fn from_credential_offer(credential_offer: CredentialOffer) -> Result<Self, WalletIssuanceError> {
        let (grant, authorization_server) = match credential_offer.grants {
            // According to the OpenID4VCI 1.0 specification:
            // (source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-2.3)
            // "When multiple grants are present, it is at the Wallet's discretion which one to use."
            //
            // Since there is no point in making the user go through an OAuth authorization flow when they have already
            // been pre-authorized, we always choose the pre-authorized code from the `CredentialOffer` if it is
            // available.
            Some(Grants {
                pre_authorized_code: Some(pre_authorized_code),
                ..
            }) => {
                if pre_authorized_code.tx_code.is_some() {
                    return Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported);
                }

                let grant = CredentialOfferGrant::GrantWithFlow {
                    flow: CredentialOfferFlow::PreAuthorizedCode {
                        pre_authorized_code: pre_authorized_code.pre_authorized_code,
                    },
                };

                (grant, pre_authorized_code.authorization_server)
            }
            Some(Grants {
                authorization_code: Some(authorization_code),
                pre_authorized_code: None,
                ..
            }) => {
                let grant = CredentialOfferGrant::GrantWithFlow {
                    flow: CredentialOfferFlow::AuthorizationCode {
                        issuer_state: authorization_code.issuer_state,
                    },
                };

                (grant, authorization_code.authorization_server)
            }
            Some(Grants {
                authorization_code: None,
                pre_authorized_code: None,
                unknown,
            }) if !unknown.is_empty() => {
                return Err(WalletIssuanceError::CredentialOfferUnknownGrants(
                    unknown.into_keys().collect(),
                ));
            }
            Some(Grants { .. }) | None => (CredentialOfferGrant::NoKnownGrant, None),
        };

        let normalized = NormalizedCredentialOffer {
            credential_issuer: credential_offer.credential_issuer,
            credential_configuration_ids: credential_offer.credential_configuration_ids,
            authorization_server,
            grant,
        };

        Ok(normalized)
    }
}

impl HttpIssuanceDiscovery {
    /// Parse a [`CredentialOffer`] from the URI or fetch it from a remote server, then convert it to a
    /// [`NormalizedCredentialOffer`].
    async fn process_credential_offer(
        &self,
        offer_uri: &Url,
    ) -> Result<NormalizedCredentialOffer, WalletIssuanceError> {
        let query = offer_uri
            .query()
            .ok_or(WalletIssuanceError::MissingCredentialOfferQuery)?;

        let offer_container = serde_urlencoded::from_str::<CredentialOfferContainer>(query)
            .map_err(WalletIssuanceError::CredentialOfferDeserialization)?;

        let credential_offer = match offer_container {
            CredentialOfferContainer::Offer { credential_offer } => *credential_offer,
            CredentialOfferContainer::Uri { credential_offer_uri } => self
                .http_client
                .get(credential_offer_uri.into_url())
                .await
                .map_err(WalletIssuanceError::CredentialOfferHttp)?,
        };

        let normalized = NormalizedCredentialOffer::from_credential_offer(credential_offer)?;

        Ok(normalized)
    }

    /// Fetch the Issuer Metadata, select an Authorization Server and fetch the OAuth server metadata from that.
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

    /// Parse or fetch the [`CredentialOffer`], fetch both the issuer and OAuth metadata and determine the flow type.
    async fn resolve_credential_offer_flow(
        &self,
        offer_uri: &Url,
    ) -> Result<
        (
            VecNonEmpty<CredentialConfigurationId>,
            CredentialOfferFlow,
            IssuerMetadata,
            AuthorizationServerMetadata,
        ),
        WalletIssuanceError,
    > {
        let credential_offer = self.process_credential_offer(offer_uri).await?;

        // TODO (PVW-5528): Use the authorization server from the Credential Offer, if provided.
        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(&credential_offer.credential_issuer).await?;

        // Collect the indices of all Credential Configuration IDs that appear in the Credential Offer, but not in the
        // Issuer Metadata. If any are missing we can use these indices to collect the owned values for returning the
        // error.
        let (credential_configs, missing_id_indices): (Vec<_>, HashSet<_>) = credential_offer
            .credential_configuration_ids
            .iter()
            .enumerate()
            .partition_map(
                |(index, id)| match issuer_metadata.credential_configurations_supported.get(id) {
                    Some(config) => Either::Left(config),
                    None => Either::Right(index),
                },
            );

        if !missing_id_indices.is_empty() {
            let missing_ids = credential_offer
                .credential_configuration_ids
                .into_iter()
                .enumerate()
                .filter_map(|(index, id)| missing_id_indices.contains(&index).then_some(id))
                .collect();

            return Err(WalletIssuanceError::MissingCredentialConfigId(missing_ids));
        }

        // According to HAIP, if the issuer requires key binding for any of its credential configurations, it MUST also
        // offer a nonce endpoint. As the wallet, we interpret this a bit more loosely and reject issuance whenever any
        // of the credential configurations offered require key binding, as the metadata may contain other
        // configurations that do not concern this particular issuance session.
        // See: https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.1-5
        if issuer_metadata.nonce_endpoint.is_none()
            && credential_configs
                .iter()
                .any(|config| config.cryptographic_binding.is_some())
        {
            return Err(WalletIssuanceError::NoNonceEndpoint);
        }

        let flow = match credential_offer.grant {
            CredentialOfferGrant::GrantWithFlow { flow } => flow,
            CredentialOfferGrant::NoKnownGrant => {
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

                CredentialOfferFlow::AuthorizationCode { issuer_state: None }
            }
        };

        Ok((
            credential_offer.credential_configuration_ids,
            flow,
            issuer_metadata,
            oauth_metadata,
        ))
    }

    #[expect(clippy::too_many_arguments, reason = "internal helper method")]
    async fn create_issuance_session(
        &self,
        pre_authorized_code: AuthorizationCode,
        credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        client_id: String,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<HttpIssuanceSession, WalletIssuanceError> {
        let message_client = HttpVcMessageClient::new(self.http_client.clone());

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: None,
            client_id: Some(client_id),
            redirect_uri: None,
        };

        HttpIssuanceSession::create(
            message_client,
            credential_configuration_ids,
            issuer_metadata,
            oauth_metadata,
            token_request,
            issuer_trust_anchors,
        )
        .await
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
    use futures::future::try_join_all;
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
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
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
        has_nonce_enpdoint: bool,
        requires_key_binding: bool,
        grant_types_supported: Option<&[&str]>,
    ) {
        // Construct issuer metadata JSON.
        let mut issuer_metadata_json = json!({
            "credential_issuer": issuer_identifier.to_string(),
            "credential_endpoint": server.url("/issuance/credential"),
            "batch_credential_endpoint": server.url("/issuance/batch_credential"),
            "credential_preview_endpoint": server.url("/issuance/credential_preview"),
            "credential_configurations_supported": {
                PID_ATTESTATION_TYPE: {
                    "format": "mso_mdoc",
                    "doctype": PID_ATTESTATION_TYPE,
                }
            },
        });
        if requires_key_binding {
            let config = &mut issuer_metadata_json["credential_configurations_supported"][PID_ATTESTATION_TYPE];
            config["cryptographic_binding_methods_supported"] = json!(["jwk"]);
            config["proof_types_supported"] = json!({
                "jwt": { "proof_signing_alg_values_supported": ["ES256"] }
            });
        }
        if has_nonce_enpdoint {
            issuer_metadata_json["nonce_endpoint"] = json!(server.url("/issuance/nonce"));
        }

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

    #[derive(Debug, Clone, Copy)]
    struct IssuerMetadataOptions {
        has_nonce_endpoint: bool,
        requires_key_binding: bool,
        has_grant_types_supported: bool,
    }

    impl Default for IssuerMetadataOptions {
        fn default() -> Self {
            Self {
                has_nonce_endpoint: true,
                requires_key_binding: true,
                has_grant_types_supported: true,
            }
        }
    }

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_httpmock_issuer(
        metadata_options: IssuerMetadataOptions,
    ) -> (MockServer, IssuerIdentifier, TrustAnchors) {
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
            metadata_options.has_nonce_endpoint,
            metadata_options.requires_key_binding,
            metadata_options
                .has_grant_types_supported
                .then_some(DEFAULT_GRANT_TYPES_SUPPORTED),
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
    async fn http_issuance_discovery_start(
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

        let (server, issuer_identifier, trust_anchor) = start_httpmock_issuer(IssuerMetadataOptions {
            has_grant_types_supported,
            ..IssuerMetadataOptions::default()
        })
        .await;

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
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
            )
            .await
            .expect("starting issuance should succeed");

        let issuance_sessions = match (scenario, flow) {
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
                // Start issuance again, this time directly expecting the Authorization Code flow.
                let second_auth_session = discovery
                    .start_authorization_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), REDIRECT_URI.clone())
                    .await
                    .expect("starting authorization code issuance should succeed");

                // Staring issuance while expecting a Pre-Authorized Code flow results in an error.
                let error = discovery
                    .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &trust_anchor)
                    .await
                    .expect_err("staring pre-authorized code issuance should fail");

                assert_matches!(error, WalletIssuanceError::CredentialOfferNoPreAuthorizedCode);

                // Continue issuance for both authorization sessions, turning them into issuance sessions.
                try_join_all(
                    [auth_session, second_auth_session]
                        .into_iter()
                        .map(async |auth_session| {
                            // Verify the auth URL points to the expected authorization endpoint and carries PAR params.
                            assert!(auth_session.auth_url().as_str().starts_with(AUTHORIZATION_ENDPOINT));
                            let auth_params: HashMap<String, String> = auth_session
                                .auth_url()
                                .query_pairs()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect();
                            assert!(auth_params.contains_key("request_uri"));
                            assert!(!auth_params.contains_key("state"));

                            // State is carried inside the PAR-stored request, not the auth URL; read it from the
                            // session.
                            let state = auth_session.state().to_owned();

                            // Simulate the authorization server redirecting back with a code and state.
                            let mut received_redirect_uri = REDIRECT_URI.clone();
                            received_redirect_uri.set_query(Some(&format!("code=fake_auth_code&state={state}")));

                            // Complete the flow — exchanges the code for a token and fetches credential previews.
                            auth_session.start_issuance(&received_redirect_uri, &trust_anchor).await
                        }),
                )
                .await
                .unwrap()
            }
            (IssuanceDiscoveryScenario::PreAuthorizedCode, IssuanceFlow::PreAuthorizedCode { issuance_session }) => {
                // Start issuance again, this time directly expecting the Pre-Authorized Code flow.
                let second_issuance_session = discovery
                    .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &trust_anchor)
                    .await
                    .expect("staring pre-authorized code issuance should succeed");

                // Staring issuance while expecting an Authorization Code flow results in an error.
                let error = discovery
                    .start_authorization_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), REDIRECT_URI.clone())
                    .await
                    .expect_err("staring authorization code issuance should fail");

                assert_matches!(error, WalletIssuanceError::CredentialOfferNoAuthorizationCode);

                // In case of the pre-authorized flow, we now have two issuance sessions.
                vec![issuance_session, second_issuance_session]
            }
            _ => {
                panic!("unexpected issuance flow type received");
            }
        };

        for issuance_session in issuance_sessions {
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
    }

    #[tokio::test]
    async fn start_missing_query() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery));
    }

    #[tokio::test]
    async fn start_deserialization_error() {
        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferDeserialization(_)));
    }

    #[tokio::test]
    async fn start_credential_offer_http_error() {
        let server = MockServer::start_async().await;

        // Construct a Credential Offer that contains an invalid URI.
        let offer_container = CredentialOfferContainer::new_uri(server.url("/does-not-exist").parse().unwrap());
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferHttp(_)));
    }

    #[tokio::test]
    async fn start_credential_offer_unknown_grants_error() {
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
            .start(
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
    async fn start_authorization_code_not_supported_error() {
        let server = MockServer::start_async().await;
        let credential_issuer = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Have the OAuth Authorization Server metadata not include "authorization_code" as a supported grant type.
        httpmock_issuer_add_metadata(&server, &credential_issuer, true, true, Some(&["implicit"])).await;

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
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::AuthorizationCodeNotSupported));
    }

    #[tokio::test]
    async fn start_credential_offer_tx_code_unsupported_error() {
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
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported));
    }

    #[tokio::test]
    async fn start_missing_credential_config_id_error() {
        let (_server, issuer_identifier, trust_anchor) = start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        // Construct a Pre-Authorized Code Credential Offer with Credential Configurations ID that are not in the Issuer
        // Metadata.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier,
            credential_configuration_ids: vec_nonempty![
                "other_id".to_string().into(),
                PID_ATTESTATION_TYPE.to_string().into(),
                "another_id".to_string().into()
            ],
            grants: Some(Grants::new_pre_authorized("fake_pre_auth_code".to_string().into())),
        };
        let offer_container = CredentialOfferContainer::new_offer(credential_offer);
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
            )
            .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::MissingCredentialConfigId(config_ids))
                if config_ids.iter().map(CredentialConfigurationId::as_ref).sorted().eq(["another_id", "other_id"])
        );
    }

    #[tokio::test]
    async fn start_no_nonce_endpoint_error() {
        // Starting issuance when the issuer metadata indicates that key binding is mandatory, yet offers no nonce
        // endpoint should fail.
        let (_server, issuer_identifier, trust_anchor) = start_httpmock_issuer(IssuerMetadataOptions {
            has_nonce_endpoint: false,
            ..IssuerMetadataOptions::default()
        })
        .await;

        let offer_container = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            "fake_pre_auth_code".to_string().into(),
        ));
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let error = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
            )
            .await
            .expect_err("starting issuance should fail");

        assert_matches!(error, WalletIssuanceError::NoNonceEndpoint);

        // When key binding is not mandatory however, the nonce endpoint can be absent.
        let (_server, issuer_identifier, trust_anchor) = start_httpmock_issuer(IssuerMetadataOptions {
            has_nonce_endpoint: false,
            requires_key_binding: false,
            ..IssuerMetadataOptions::default()
        })
        .await;

        let offer_container = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![PID_ATTESTATION_TYPE.to_string().into()],
            "fake_pre_auth_code".to_string().into(),
        ));
        let query = serde_urlencoded::to_string(&offer_container).unwrap();
        let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

        let _flow = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
            )
            .await
            .expect("starting issuance should succeed");
    }
}
