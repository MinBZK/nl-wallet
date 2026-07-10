use std::collections::HashSet;

use crypto::trust_anchor::TrustAnchors;
use http_utils::reqwest::HttpClient;
use itertools::Either;
use itertools::Itertools;
use jwt::DEFAULT_VALIDATIONS;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithX5c;
use jwt::wia::WIA_CLIENT_AUTH_METHOD;
use url::Url;
use utils::generator::TimeGenerator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;
use wscd::wscd::WiaClient;

use super::AuthorizationSession;
use super::IssuanceDiscovery;
use super::IssuanceFlow;
use super::WalletIssuanceError;
use super::authorization::HttpAuthorizationSession;
use super::authorization_endpoints::AuthorizationEndpoints;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
use crate::credential_offer::CredentialOffer;
use crate::credential_offer::CredentialOfferContainer;
use crate::credential_offer::Grants;
use crate::issuer_identifier::IssuerIdentifier;
use crate::jose::JwsAlgorithm;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::SignedIssuerMetadataPayload;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::metadata::well_known;
use crate::metadata::well_known::WellKnownPath;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;

pub struct HttpIssuanceDiscovery {
    http_client: HttpClient,
}

impl HttpIssuanceDiscovery {
    pub fn new(http_client: HttpClient) -> Self {
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
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError> {
        let (credential_configurations, credential_issuer, issuer_endpoints, flow) = self
            .resolve_credential_offer_flow(offer_uri, wrpac_trust_anchors)
            .await?;

        let issuance_flow = match flow {
            CredentialOfferFlow::AuthorizationCode {
                issuer_state,
                auth_endpoints,
                authorization_server,
            } => {
                let authorization_session = HttpAuthorizationSession::create(
                    self.http_client.clone(),
                    credential_configurations,
                    credential_issuer,
                    issuer_endpoints,
                    auth_endpoints,
                    client_id,
                    redirect_uri,
                    issuer_state,
                    wia_client,
                    authorization_server,
                )
                .await?;

                IssuanceFlow::AuthorizationCode { authorization_session }
            }
            CredentialOfferFlow::PreAuthorizedCode {
                pre_authorized_code,
                token_endpoint,
                authorization_server,
            } => {
                let issuance_session = self
                    .create_issuance_session(
                        pre_authorized_code,
                        credential_configurations,
                        credential_issuer,
                        issuer_endpoints,
                        &token_endpoint,
                        wia_client,
                        &authorization_server,
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
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Authorization, WalletIssuanceError> {
        let (credential_configurations, credential_identifier, issuer_endpoints, flow) = self
            .resolve_credential_offer_flow(offer_uri, wrpac_trust_anchors)
            .await?;

        let CredentialOfferFlow::AuthorizationCode {
            issuer_state,
            auth_endpoints,
            authorization_server,
        } = flow
        else {
            return Err(WalletIssuanceError::CredentialOfferNoAuthorizationCode);
        };

        HttpAuthorizationSession::create(
            self.http_client.clone(),
            credential_configurations,
            credential_identifier,
            issuer_endpoints,
            auth_endpoints,
            client_id,
            redirect_uri,
            issuer_state,
            wia_client,
            authorization_server,
        )
        .await
    }

    async fn start_pre_authorized_code_flow(
        &self,
        offer_uri: &Url,
        issuer_trust_anchors: &TrustAnchors,
        wia_client: &impl WiaClient,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let (credential_configurations, credential_identifier, issuer_endpoints, flow) = self
            .resolve_credential_offer_flow(offer_uri, wrpac_trust_anchors)
            .await?;

        let CredentialOfferFlow::PreAuthorizedCode {
            pre_authorized_code,
            token_endpoint,
            authorization_server,
        } = flow
        else {
            return Err(WalletIssuanceError::CredentialOfferNoPreAuthorizedCode);
        };

        self.create_issuance_session(
            pre_authorized_code,
            credential_configurations,
            credential_identifier,
            issuer_endpoints,
            &token_endpoint,
            wia_client,
            &authorization_server,
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
    credential_configuration_ids: VecNonEmptyUnique<CredentialConfigurationId>,
    authorization_server: Option<IssuerIdentifier>,
    grant: CredentialOfferGrant,
}

#[derive(Debug)]
enum CredentialOfferGrant {
    AuthorizationCode { issuer_state: Option<String> },
    PreAuthorizedCode { pre_authorized_code: AuthorizationCode },
    NoKnownGrant,
}

#[derive(Debug)]
enum CredentialOfferFlow {
    AuthorizationCode {
        issuer_state: Option<String>,
        authorization_server: IssuerIdentifier,
        auth_endpoints: AuthorizationEndpoints,
    },
    PreAuthorizedCode {
        pre_authorized_code: AuthorizationCode,
        authorization_server: IssuerIdentifier,
        token_endpoint: Url,
    },
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

                let grant = CredentialOfferGrant::PreAuthorizedCode {
                    pre_authorized_code: pre_authorized_code.pre_authorized_code,
                };

                (grant, pre_authorized_code.authorization_server)
            }
            Some(Grants {
                authorization_code: Some(authorization_code),
                pre_authorized_code: None,
                ..
            }) => {
                let grant = CredentialOfferGrant::AuthorizationCode {
                    issuer_state: authorization_code.issuer_state,
                };

                (grant, authorization_code.authorization_server)
            }
            Some(Grants {
                authorization_code: _,
                pre_authorized_code: _,
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

impl CredentialOfferFlow {
    /// Determine which flow to use based on the preferred Grant present in the Credential Offer, combined with the
    /// OAuth metadata, while extracting the relevant portions of that metadata.
    fn try_from_offer_grant(
        offer_grant: CredentialOfferGrant,
        oauth_metadata: AuthorizationServerMetadata,
    ) -> Result<Self, WalletIssuanceError> {
        let flow = match offer_grant {
            CredentialOfferGrant::AuthorizationCode { issuer_state } => {
                let authorization_server = oauth_metadata.issuer.clone();

                let auth_endpoints = oauth_metadata
                    .try_into()
                    .map_err(WalletIssuanceError::AuthorizationEndpoints)?;

                Self::AuthorizationCode {
                    issuer_state,
                    auth_endpoints,
                    authorization_server,
                }
            }
            CredentialOfferGrant::PreAuthorizedCode { pre_authorized_code } => Self::PreAuthorizedCode {
                pre_authorized_code,
                authorization_server: oauth_metadata.issuer,
                token_endpoint: oauth_metadata.token_endpoint,
            },
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

                let authorization_server = oauth_metadata.issuer.clone();

                let auth_endpoints = oauth_metadata
                    .try_into()
                    .map_err(WalletIssuanceError::AuthorizationEndpoints)?;

                Self::AuthorizationCode {
                    issuer_state: None,
                    authorization_server,
                    auth_endpoints,
                }
            }
        };

        Ok(flow)
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

        let offer_container = serde_qs::from_str::<CredentialOfferContainer>(query)
            .map_err(WalletIssuanceError::CredentialOfferDeserialization)?;

        let credential_offer = match offer_container {
            CredentialOfferContainer::CredentialOffer(credential_offer) => *credential_offer,
            CredentialOfferContainer::CredentialOfferUri(credential_offer_uri) => self
                .http_client
                .get_json(credential_offer_uri.into_url())
                .await
                .map_err(WalletIssuanceError::CredentialOfferHttp)?,
        };

        let normalized = NormalizedCredentialOffer::from_credential_offer(credential_offer)?;

        Ok(normalized)
    }

    /// Fetch the Issuer Metadata, select an Authorization Server and fetch the OAuth server metadata from that.
    async fn fetch_metadata(
        &self,
        credential_offer: &NormalizedCredentialOffer,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<(IssuerMetadata, AuthorizationServerMetadata), WalletIssuanceError> {
        let issuer_metadata_jwt: UnverifiedJwt<SignedIssuerMetadataPayload, HeaderWithX5c> = self
            .http_client
            .get_jwt(WellKnownPath::CredentialIssuer.url(&credential_offer.credential_issuer))
            .await
            .map_err(WalletIssuanceError::CredentialIssuerMetadataHttp)?
            .parse()?;

        let issuer_metadata_payload = issuer_metadata_jwt
            .into_verified_against_trust_anchors(wrpac_trust_anchors, &TimeGenerator, None, &DEFAULT_VALIDATIONS)
            .map_err(WalletIssuanceError::CredentialIssuerMetadataVerify)?
            .into_payload();
        if *issuer_metadata_payload.sub != credential_offer.credential_issuer {
            return Err(WalletIssuanceError::CredentialIssuerMetadataIdentifierMismatch {
                expected: Box::new(credential_offer.credential_issuer.clone()),
                received: Box::new(issuer_metadata_payload.sub.into_owned()),
            });
        }

        let issuer_metadata = issuer_metadata_payload.metadata.into_owned();
        if issuer_metadata.credential_issuer != credential_offer.credential_issuer {
            return Err(WalletIssuanceError::CredentialIssuerMetadataIdentifierMismatch {
                expected: Box::new(credential_offer.credential_issuer.clone()),
                received: Box::new(issuer_metadata.credential_issuer),
            });
        }

        let metadata_auth_servers = issuer_metadata.authorization_servers();
        let authorization_server = match credential_offer.authorization_server.as_ref() {
            Some(authorization_server) => {
                // If the Credential Offer contains an Authorization Server, it must match one of the entries in the
                // Issuer Metadata.
                if !metadata_auth_servers.as_ref().contains(&authorization_server) {
                    return Err(WalletIssuanceError::AuthorizationServerMismatch(
                        Box::new(authorization_server.clone()),
                        Box::new(metadata_auth_servers.nonempty_iter().copied().cloned().collect()),
                    ));
                }

                authorization_server
            }
            None => {
                // Otherwise, choose one at random from the list Authorization Servers in order to be a good client and
                // load-balance requests between them.
                metadata_auth_servers.choose()
            }
        };

        let oauth_metadata: AuthorizationServerMetadata = well_known::fetch_well_known(
            &self.http_client,
            authorization_server,
            WellKnownPath::OauthAuthorizationServer,
        )
        .await
        .map_err(WalletIssuanceError::OauthDiscovery)?;

        Ok((issuer_metadata, oauth_metadata))
    }

    /// Parse or fetch the [`CredentialOffer`], fetch both the issuer and OAuth metadata and determine the flow type.
    async fn resolve_credential_offer_flow(
        &self,
        offer_uri: &Url,
        wrpac_trust_anchors: &TrustAnchors,
    ) -> Result<
        (
            VecNonEmpty<(CredentialConfigurationId, CredentialConfiguration)>,
            IssuerIdentifier,
            IssuerEndpoints,
            CredentialOfferFlow,
        ),
        WalletIssuanceError,
    > {
        let credential_offer = self.process_credential_offer(offer_uri).await?;

        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(&credential_offer, wrpac_trust_anchors).await?;

        Self::check_client_attestation_metadata(&oauth_metadata)?;

        let IssuerMetadata {
            credential_issuer,
            endpoints: issuer_endpoints,
            mut credential_configurations_supported,
            ..
        } = issuer_metadata;

        // Collect the indices of all Credential Configuration IDs that appear in the Credential Offer, but not in the
        // Issuer Metadata. If any are missing we can use these indices to collect the owned values for returning the
        // error.
        let (credential_configs, missing_ids): (Vec<_>, HashSet<_>) = credential_offer
            .credential_configuration_ids
            .into_iter()
            .enumerate()
            .partition_map(
                move |(_index, id)| match credential_configurations_supported.remove(&id) {
                    Some(config) => Either::Left((id, config)),
                    None => Either::Right(id),
                },
            );

        if !missing_ids.is_empty() {
            return Err(WalletIssuanceError::MissingCredentialConfigId(missing_ids));
        }
        let credential_configs =
            VecNonEmpty::try_from(credential_configs).expect("credential_configuration_ids is VecNonEmpty");

        // According to HAIP, if the issuer requires key binding for any of its credential configurations, it MUST also
        // offer a nonce endpoint. As the wallet, we interpret this a bit more loosely and reject issuance whenever any
        // of the credential configurations offered require key binding, as the metadata may contain other
        // configurations that do not concern this particular issuance session.
        // See: https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.1-5
        if issuer_endpoints.nonce_endpoint.is_none()
            && credential_configs
                .iter()
                .any(|(_, config)| config.cryptographic_binding.is_some())
        {
            return Err(WalletIssuanceError::NoNonceEndpoint);
        }

        let flow = CredentialOfferFlow::try_from_offer_grant(credential_offer.grant, oauth_metadata)?;

        Ok((credential_configs, credential_issuer, issuer_endpoints, flow))
    }

    fn check_client_attestation_metadata(
        oauth_metadata: &AuthorizationServerMetadata,
    ) -> Result<(), WalletIssuanceError> {
        if !oauth_metadata
            .token_endpoint_auth_methods_supported
            .as_ref()
            .is_some_and(|auth_methods| auth_methods.contains(WIA_CLIENT_AUTH_METHOD))
        {
            return Err(WalletIssuanceError::NoAttestationBasedClientAuthSupport);
        }

        if !oauth_metadata
            .client_attestation_signing_alg_values_supported
            .as_ref()
            .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
        {
            return Err(WalletIssuanceError::ClientAttestationSigningAlgNotSupported(
                oauth_metadata.client_attestation_signing_alg_values_supported.clone(),
            ));
        }

        if !oauth_metadata
            .client_attestation_pop_signing_alg_values_supported
            .as_ref()
            .is_some_and(|algs| algs.contains(&JwsAlgorithm::ES256))
        {
            return Err(WalletIssuanceError::ClientAttestationPopSigningAlgNotSupported(
                oauth_metadata
                    .client_attestation_pop_signing_alg_values_supported
                    .clone(),
            ));
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments, reason = "internal helper method")]
    async fn create_issuance_session(
        &self,
        pre_authorized_code: AuthorizationCode,
        credential_configurations: VecNonEmpty<(CredentialConfigurationId, CredentialConfiguration)>,
        credential_issuer: IssuerIdentifier,
        issuer_endpoints: IssuerEndpoints,
        token_endpoint: &Url,
        wia_client: &impl WiaClient,
        authorization_server: &IssuerIdentifier,
        issuer_trust_anchors: &TrustAnchors,
    ) -> Result<HttpIssuanceSession, WalletIssuanceError> {
        let message_client = HttpVcMessageClient::new(self.http_client.clone());

        let token_request = TokenRequest::new_pre_authorized(pre_authorized_code);

        HttpIssuanceSession::create(
            message_client,
            credential_configurations,
            credential_issuer,
            issuer_endpoints,
            token_endpoint,
            token_request,
            wia_client,
            authorization_server,
            issuer_trust_anchors,
        )
        .await
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::num::NonZeroU8;
    use std::sync::LazyLock;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::credential_format::Format;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::DateTime;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use futures::future::try_join_all;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpClient;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use itertools::Itertools;
    use jwt::SignedJwt;
    use jwt::UnverifiedJwt;
    use jwt::error::JwtVerifyError;
    use jwt::error::JwtX5cVerifyError;
    use jwt::wia::WIA_CLIENT_AUTH_METHOD;
    use rstest::rstest;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use serde_json::json;
    use url::Url;
    use utils::date_time_seconds::DateTimeSeconds;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::mock_remote::MockWiaClient;

    use super::HttpIssuanceDiscovery;
    use super::IssuanceDiscovery;
    use crate::credential_offer::CredentialOffer;
    use crate::credential_offer::CredentialOfferContainer;
    use crate::credential_offer::GrantPreAuthorizedCode;
    use crate::credential_offer::Grants;
    use crate::credential_offer::PreAuthTransactionCode;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::jose::JwsAlgorithm;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::issuer_metadata::SignedIssuerMetadataPayload;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::preview::CredentialPreviewResponse;
    use crate::token::CredentialPreview;
    use crate::token::TokenResponse;
    use crate::wallet_issuance::AuthorizationSession;
    use crate::wallet_issuance::IssuanceFlow;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::authorization::HttpAuthorizationSession;
    use crate::wallet_issuance::issuance_session::HttpIssuanceSession;

    static CONFIG_ID: LazyLock<CredentialConfigurationId> = LazyLock::new(|| "pid".to_string().into());

    const DEFAULT_GRANT_TYPES_SUPPORTED: &[&str] = &[
        "authorization_code",
        "urn:ietf:params:oauth:grant-type:pre-authorized_code",
    ];
    static REDIRECT_URI: LazyLock<Url> = LazyLock::new(|| "https://wallet.example.com/callback".parse().unwrap());
    const AUTHORIZATION_ENDPOINT: &str = "https://auth.example.com/authorize";

    /// Creates a method that converts issuer metadata into a valid signed metadata
    fn default_signed_metadata<'a>() -> impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a>
    {
        custom_signed_metadata(None, None, false)
    }

    /// Creates a method that converts issuer metadata into signed metadata with customization
    fn custom_signed_metadata<'a>(
        custom_jwt_identifier: Option<IssuerIdentifier>,
        custom_metadata_identifier: Option<IssuerIdentifier>,
        expired: bool,
    ) -> impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a> {
        move |issuer_identifier, mut issuer_metadata| -> SignedIssuerMetadataPayload {
            let iat = DateTimeSeconds::new(DateTime::from_timestamp_secs(12345678).unwrap());
            if let Some(custom) = custom_metadata_identifier {
                issuer_metadata.credential_issuer = custom;
            };
            SignedIssuerMetadataPayload {
                metadata: Cow::Owned(issuer_metadata),

                iss: None,
                sub: Cow::Owned(custom_jwt_identifier.unwrap_or(issuer_identifier)),
                iat,
                exp: if expired { Some(iat) } else { None },
            }
        }
    }

    async fn httpmock_issuer_add_metadata<'a>(
        server: &MockServer,
        has_nonce_enpdoint: bool,
        requires_key_binding: bool,
        grant_types_supported: Option<&[&str]>,
        has_client_attestation_support: bool,
        to_signed_metadata: impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a>,
    ) -> (IssuerIdentifier, TrustAnchors) {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let wrpac_keypair = ca.generate_wrpac_issuer_mock().unwrap();

        let issuer_identifier = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct issuer metadata JSON.
        let mut issuer_metadata_json = json!({
            "credential_issuer": issuer_identifier.to_string(),
            "credential_endpoint": server.url("/issuance/credential"),
            "batch_credential_endpoint": server.url("/issuance/batch_credential"),
            "credential_preview_endpoint": server.url("/issuance/credential_preview"),
            "credential_configurations_supported": {
                CONFIG_ID.as_ref(): {
                    "format": "mso_mdoc",
                    "doctype": PID_ATTESTATION_TYPE,
                    "scope": "pid_scope",
                    "type_metadata_uri": issuer_identifier
                                            .as_issuer_url()
                                            .join_issuer_url("/issuance/type_metadata")
                                            .join_config_id(&CONFIG_ID),
                }
            },
        });
        if requires_key_binding {
            let config = &mut issuer_metadata_json["credential_configurations_supported"][CONFIG_ID.as_ref()];
            config["cryptographic_binding_methods_supported"] = json!(["jwk"]);
            config["proof_types_supported"] = json!({
                "jwt": { "proof_signing_alg_values_supported": ["ES256"] }
            });
        }
        if has_nonce_enpdoint {
            issuer_metadata_json["nonce_endpoint"] = json!(server.url("/issuance/nonce"));
        }

        let issuer_metadata = serde_json::from_value(issuer_metadata_json).unwrap();
        let signed_issuer_metadata_payload = to_signed_metadata(issuer_identifier.clone(), issuer_metadata);
        let signed_issuer_metadata = SignedJwt::sign_with_certificate(&signed_issuer_metadata_payload, &wrpac_keypair)
            .await
            .unwrap();

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
        if has_client_attestation_support {
            oauth_metadata_json["token_endpoint_auth_methods_supported"] = json!([WIA_CLIENT_AUTH_METHOD]);
            oauth_metadata_json["client_attestation_signing_alg_values_supported"] = json!(["ES256"]);
            oauth_metadata_json["client_attestation_pop_signing_alg_values_supported"] = json!(["ES256"]);
        }

        server
            .mock_async(|when, then| {
                when.method(GET).path("/.well-known/openid-credential-issuer");

                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), "application/jwt")
                    .body(UnverifiedJwt::from(signed_issuer_metadata).serialization());
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

        (issuer_identifier, TrustAnchors::from(&ca))
    }

    #[derive(Debug, Clone, Copy)]
    struct IssuerMetadataOptions {
        has_nonce_endpoint: bool,
        requires_key_binding: bool,
        has_grant_types_supported: bool,
        has_client_attestation_support: bool,
    }

    impl Default for IssuerMetadataOptions {
        fn default() -> Self {
            Self {
                has_nonce_endpoint: true,
                requires_key_binding: true,
                has_grant_types_supported: true,
                has_client_attestation_support: true,
            }
        }
    }

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_httpmock_issuer(
        metadata_options: IssuerMetadataOptions,
    ) -> (MockServer, IssuerIdentifier, TrustAnchors, TrustAnchors) {
        let server = MockServer::start_async().await;

        // Create CA and issuer certificate for the credential preview.
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair =
            generate_pid_issuer_mock_with_registration(&issuer_ca, &IssuerRegistration::new_mock()).unwrap();

        // Create type metadata for the credential preview.
        let (_, _, type_metadata_documents) = TypeMetadataDocuments::from_single_example(
            TypeMetadata::example_with_claim_name(PID_ATTESTATION_TYPE, "family_name"),
        );

        let credential_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let preview = CredentialPreview {
            config_id: CONFIG_ID.clone(),
            format: Format::MsoMdoc,
            batch_size: NonZeroU8::new(4).unwrap(),
            credential_payload,
            issuer_certificate: issuance_keypair.certificate().clone(),
        };

        let preview_response = CredentialPreviewResponse {
            credential_previews: vec_nonempty![preview],
        };

        let token_response = TokenResponse::new("mock_access_token".to_string().into());

        let (issuer_identifier, wrpac_trust_anchors) = httpmock_issuer_add_metadata(
            &server,
            metadata_options.has_nonce_endpoint,
            metadata_options.requires_key_binding,
            metadata_options
                .has_grant_types_supported
                .then_some(DEFAULT_GRANT_TYPES_SUPPORTED),
            metadata_options.has_client_attestation_support,
            default_signed_metadata(),
        )
        .await;

        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/issuance/type_metadata/{}", CONFIG_ID.as_ref()));
                then.status(200)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(json!(type_metadata_documents));
            })
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

        (
            server,
            issuer_identifier,
            TrustAnchors::from(&issuer_ca),
            wrpac_trust_anchors,
        )
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

        let (server, issuer_identifier, issuer_trust_anchors, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions {
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
            credential_configuration_ids: vec_nonempty![CONFIG_ID.clone()].into(),
            grants,
        };

        // If the Credential Offer is by reference, have the mock issuance server serve it. Construct the Credential
        // Offer URL based on this.
        let offer_url = if is_by_reference {
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
        }
        .to_credential_offer_url();

        // Start issuance based on this Credential Offer URL.
        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let flow = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &issuer_trust_anchors,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
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
                    .start_authorization_code_flow(
                        &offer_url,
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        REDIRECT_URI.clone(),
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                    )
                    .await
                    .expect("starting authorization code issuance should succeed");

                // Staring issuance while expecting a Pre-Authorized Code flow results in an error.
                let error = discovery
                    .start_pre_authorized_code_flow(
                        &offer_url,
                        &issuer_trust_anchors,
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                    )
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
                            auth_session
                                .start_issuance(&received_redirect_uri, &issuer_trust_anchors, &MockWiaClient::new())
                                .await
                        }),
                )
                .await
                .unwrap()
            }
            (IssuanceDiscoveryScenario::PreAuthorizedCode, IssuanceFlow::PreAuthorizedCode { issuance_session }) => {
                // Start issuance again, this time directly expecting the Pre-Authorized Code flow.
                let second_issuance_session = discovery
                    .start_pre_authorized_code_flow(
                        &offer_url,
                        &issuer_trust_anchors,
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                    )
                    .await
                    .expect("staring pre-authorized code issuance should succeed");

                // Staring issuance while expecting an Authorization Code flow results in an error.
                let error = discovery
                    .start_authorization_code_flow(
                        &offer_url,
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        REDIRECT_URI.clone(),
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                    )
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
            assert_eq!(issuance_session.credential_previews().len(), 1);
            assert_eq!(
                issuance_session.credential_previews()[0]
                    .credential_payload
                    .attestation_type,
                PID_ATTESTATION_TYPE
            );
        }
    }

    #[tokio::test]
    async fn start_missing_query() {
        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery));
    }

    #[tokio::test]
    async fn start_deserialization_error() {
        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferDeserialization(_)));
    }

    #[tokio::test]
    async fn start_credential_offer_http_error() {
        let server = MockServer::start_async().await;

        // Construct a Credential Offer that contains an invalid URI.
        let offer_url =
            CredentialOfferContainer::new_uri(server.url("/does-not-exist").parse().unwrap()).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
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
            "credential_configuration_ids": [CONFIG_ID.as_ref()],
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

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
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

        // Have the OAuth Authorization Server metadata not include "authorization_code" as a supported grant type.
        let (credential_issuer, wrpac_trust_anchors) = httpmock_issuer_add_metadata(
            &server,
            true,
            true,
            Some(&["implicit"]),
            true,
            default_signed_metadata(),
        )
        .await;

        // Construct a Credential Offer that contains no grants.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![CONFIG_ID.clone()].into(),
            grants: None,
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
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
            credential_configuration_ids: vec_nonempty![CONFIG_ID.clone()].into(),
            grants: Some(Grants {
                pre_authorized_code: Some(GrantPreAuthorizedCode {
                    pre_authorized_code: "code".to_string().into(),
                    tx_code: Some(PreAuthTransactionCode::default()),
                    authorization_server: None,
                }),
                ..Grants::default()
            }),
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
                &TrustAnchors::empty(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported));
    }

    async fn start_check_metadata<'a>(
        to_signed_metadata: impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a>,
        use_trust_anchors: bool,
    ) -> (
        IssuerIdentifier,
        Result<IssuanceFlow<HttpAuthorizationSession, HttpIssuanceSession>, WalletIssuanceError>,
    ) {
        let server = MockServer::start_async().await;

        // Setup simple metadata server
        let (credential_issuer, wrpac_trust_anchors) =
            httpmock_issuer_add_metadata(&server, false, false, None, true, to_signed_metadata).await;

        // Construct a Credential Offer
        let credential_offer = CredentialOffer {
            credential_issuer: credential_issuer.clone(),
            credential_configuration_ids: vec_nonempty![CONFIG_ID.clone()].into(),
            grants: None,
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        // Start discovery
        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());
        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &TrustAnchors::empty(),
                &MockWiaClient::new(),
                &if use_trust_anchors {
                    wrpac_trust_anchors
                } else {
                    TrustAnchors::empty()
                },
            )
            .await;
        (credential_issuer, result)
    }

    #[tokio::test]
    async fn start_untrusted_metadata() {
        let (_, result) = start_check_metadata(default_signed_metadata(), false).await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataVerify(
                JwtX5cVerifyError::CertificateValidation(_)
            ))
        );
    }

    #[tokio::test]
    async fn start_expired_metadata() {
        let (_, result) = start_check_metadata(custom_signed_metadata(None, None, true), true).await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataVerify(
                JwtX5cVerifyError::JwtVerify(JwtVerifyError::Validation(_))
            ))
        );
    }

    #[tokio::test]
    async fn start_metadata_issuer_mismatch() {
        let different_identifier = IssuerIdentifier::try_new("https://example.com/totally_different".into()).unwrap();
        let (offered_identifier, result) = start_check_metadata(
            custom_signed_metadata(Some(different_identifier.clone()), None, false),
            true,
        )
        .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataIdentifierMismatch{
                expected, received
            }) if *expected == offered_identifier && *received == different_identifier
        );
    }

    #[tokio::test]
    async fn start_metadata_jwt_issuer_mismatch() {
        let different_identifier = IssuerIdentifier::try_new("https://example.com/totally_different".into()).unwrap();
        let (offered_identifier, result) = start_check_metadata(
            custom_signed_metadata(None, Some(different_identifier.clone()), false),
            true,
        )
        .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataIdentifierMismatch{
                expected, received
            }) if *expected == offered_identifier && *received == different_identifier
        );
    }

    #[tokio::test]
    async fn start_authorization_server_mismatch_error() {
        let (_server, issuer_identifier, issuer_trust_anchors, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        // Construct a Pre-Authorized Code Credential Offer with an unknown Authorization Server.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier.clone(),
            credential_configuration_ids: vec_nonempty![CONFIG_ID.clone()].into(),
            grants: Some(Grants {
                pre_authorized_code: Some(GrantPreAuthorizedCode {
                    pre_authorized_code: "code".to_string().into(),
                    tx_code: None,
                    authorization_server: Some("https://auth.example.com".parse().unwrap()),
                }),
                ..Grants::default()
            }),
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &issuer_trust_anchors,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
            )
            .await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::AuthorizationServerMismatch(auth_server, metadata_auth_servers))
                if auth_server.as_ref().as_ref() == "https://auth.example.com" &&
                    metadata_auth_servers.iter().eq([&issuer_identifier])
        );
    }

    #[tokio::test]
    async fn start_missing_credential_config_id_error() {
        let (_server, issuer_identifier, issuer_trust_anchors, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        // Construct a Pre-Authorized Code Credential Offer with Credential Configurations ID that are not in the Issuer
        // Metadata.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier,
            credential_configuration_ids: vec_nonempty![
                "other_id".to_string().into(),
                CONFIG_ID.clone(),
                "another_id".to_string().into()
            ]
            .into(),
            grants: Some(Grants::new_pre_authorized("fake_pre_auth_code".to_string().into())),
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let result = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &issuer_trust_anchors,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
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
        let (_server, issuer_identifier, issuer_trust_anchors, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions {
                has_nonce_endpoint: false,
                ..IssuerMetadataOptions::default()
            })
            .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let error = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &issuer_trust_anchors,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
            )
            .await
            .expect_err("starting issuance should fail");

        assert_matches!(error, WalletIssuanceError::NoNonceEndpoint);

        // When key binding is not mandatory however, the nonce endpoint can be absent.
        let (_server, issuer_identifier, issuer_trust_anchors, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions {
                has_nonce_endpoint: false,
                requires_key_binding: false,
                ..IssuerMetadataOptions::default()
            })
            .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let _flow = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &issuer_trust_anchors,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
            )
            .await
            .expect("starting issuance should succeed");
    }

    #[tokio::test]
    async fn start_no_attestation_based_client_auth_support_error() {
        // Starting issuance when the Authorization Server metadata does not advertise support for
        // Attestation-Based Client Authentication should fail.
        let (_server, issuer_identifier, trust_anchor, wrpac_trust_anchors) =
            start_httpmock_issuer(IssuerMetadataOptions {
                has_client_attestation_support: false,
                ..IssuerMetadataOptions::default()
            })
            .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap());

        let error = discovery
            .start(
                &offer_url,
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
                &trust_anchor,
                &MockWiaClient::new(),
                &wrpac_trust_anchors,
            )
            .await
            .expect_err("starting issuance should fail");

        assert_matches!(error, WalletIssuanceError::NoAttestationBasedClientAuthSupport);
    }

    /// Returns [`AuthorizationServerMetadata`] that fully supports Attestation-Based Client Authentication.
    fn oauth_metadata_with_client_attestation_support() -> AuthorizationServerMetadata {
        let mut metadata = AuthorizationServerMetadata::new(
            "https://issuer.example.com".parse().unwrap(),
            "https://issuer.example.com/token".parse().unwrap(),
        );
        metadata.token_endpoint_auth_methods_supported = Some([WIA_CLIENT_AUTH_METHOD.to_string()].into());
        metadata.client_attestation_signing_alg_values_supported = Some([JwsAlgorithm::ES256].into());
        metadata.client_attestation_pop_signing_alg_values_supported = Some([JwsAlgorithm::ES256].into());

        metadata
    }

    #[test]
    fn check_client_attestation_metadata_ok() {
        let metadata = oauth_metadata_with_client_attestation_support();

        HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata)
            .expect("client attestation metadata should be accepted");
    }

    #[test]
    fn check_client_attestation_metadata_no_token_endpoint_auth_methods() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.token_endpoint_auth_methods_supported = None;

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_wia_auth_method_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.token_endpoint_auth_methods_supported = Some(["client_secret_basic".to_string()].into());

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::NoAttestationBasedClientAuthSupport)
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_signing_alg_values() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_signing_alg_values_supported = None;

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::ClientAttestationSigningAlgNotSupported(None))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_signing_alg_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::ClientAttestationSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }

    #[test]
    fn check_client_attestation_metadata_no_pop_signing_alg_values() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_pop_signing_alg_values_supported = None;

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::ClientAttestationPopSigningAlgNotSupported(None))
        );
    }

    #[test]
    fn check_client_attestation_metadata_es256_pop_signing_alg_not_supported() {
        let mut metadata = oauth_metadata_with_client_attestation_support();
        metadata.client_attestation_pop_signing_alg_values_supported =
            Some([JwsAlgorithm::Other("RS256".to_string())].into());

        assert_matches!(
            HttpIssuanceDiscovery::check_client_attestation_metadata(&metadata),
            Err(WalletIssuanceError::ClientAttestationPopSigningAlgNotSupported(Some(algs)))
                if algs.iter().eq([&JwsAlgorithm::Other("RS256".to_string())])
        );
    }
}
