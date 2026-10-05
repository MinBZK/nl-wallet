use std::collections::HashMap;
use std::num::NonZeroU8;
use std::sync::Arc;

use crypto::trust_anchor::TrustAnchors;
use crypto::x509::crl::CertificateCrlVerifier;
use crypto::x509::crl::CrlFetcher;
use crypto::x509::crl::HttpCrlFetcher;
use http_utils::reqwest::HttpClient;
use itertools::Either;
use itertools::Itertools;
use jwt::DEFAULT_VALIDATION;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithX5c;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::metadata::well_known::WellKnownMetadata;
use oauth::token::AuthorizationCode;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::reqwest::HttpStatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;
use url::Url;
use utils::generator::TimeGenerator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmptyUnique;
use wscd::wia::WiaClient;

use super::AuthorizationSession;
use super::IssuanceDiscovery;
use super::IssuanceDiscoveryParameters;
use super::IssuanceFlow;
use super::WalletIssuanceError;
use super::authorization::HttpAuthorizationSession;
use super::authorization_endpoints::AuthorizationEndpoints;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
use super::issuer_registration::IssuerRegistration;
use crate::client_auth::ClientAttestationChallengeMechanism;
use crate::client_auth::check_client_attestation_metadata;
use crate::credential_offer::CredentialOffer;
use crate::credential_offer::CredentialOfferContainer;
use crate::credential_offer::Grants;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::metadata::issuer_metadata::IssuerInfo;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::SignedIssuerMetadataPayload;
use crate::metadata::oauth_metadata::IssuerAuthorizationServerMetadata;
use crate::registration_certificate::RegistrationCertificateError;
use crate::registration_certificate::validate_registration_certificate;
use crate::token::VciTokenRequest;
use crate::wallet_issuance::CredentialSelection;

const BATCH_SIZE_MAX: NonZeroU8 = NonZeroU8::MAX;

pub struct HttpIssuanceDiscovery<F = HttpCrlFetcher, C = HttpStatusListClient> {
    http_client: HttpClient,
    crl_verifier: CertificateCrlVerifier<F>,
    wrprc_revocation_verifier: RevocationVerifier<C>,
}

impl<F, C> HttpIssuanceDiscovery<F, C>
where
    C: StatusListClient,
{
    pub fn new(http_client: HttpClient, crl_verifier: CertificateCrlVerifier<F>, status_list_client: C) -> Self {
        Self {
            http_client,
            crl_verifier,
            wrprc_revocation_verifier: RevocationVerifier::new_with_defaults(
                Arc::new(status_list_client),
                TimeGenerator,
            ),
        }
    }
}

impl<F, C> IssuanceDiscovery for HttpIssuanceDiscovery<F, C>
where
    F: CrlFetcher,
    C: StatusListClient,
{
    type Authorization = HttpAuthorizationSession;
    type Issuance = HttpIssuanceSession;

    async fn start<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError>
    where
        W: WiaClient,
    {
        let IssuanceDiscoveryParameters {
            offer_uri,
            selection,
            wia_client,
            wrpac_trust_anchors,
            wrprc_trust_anchors,
        } = common_parameters;

        let (credential_configurations, credential_issuer, issuer_endpoints, issuer_registration, batch_size, flow) =
            self.resolve_credential_offer_flow(offer_uri, selection, wrpac_trust_anchors, wrprc_trust_anchors)
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
                    issuer_registration,
                    batch_size,
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
                challenge_endpoint,
            } => {
                let issuance_session = self
                    .create_issuance_session(
                        pre_authorized_code,
                        credential_configurations,
                        credential_issuer,
                        issuer_endpoints,
                        issuer_registration,
                        batch_size,
                        token_endpoint,
                        challenge_endpoint,
                        wia_client,
                        &authorization_server,
                    )
                    .await?;

                IssuanceFlow::PreAuthorizedCode { issuance_session }
            }
        };

        Ok(issuance_flow)
    }

    async fn start_authorization_code_flow<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError>
    where
        W: WiaClient,
    {
        let IssuanceDiscoveryParameters {
            offer_uri,
            selection,
            wia_client,
            wrpac_trust_anchors,
            wrprc_trust_anchors,
        } = common_parameters;

        let (credential_configurations, credential_identifier, issuer_endpoints, issuer_registration, batch_size, flow) =
            self.resolve_credential_offer_flow(offer_uri, selection, wrpac_trust_anchors, wrprc_trust_anchors)
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
            issuer_registration,
            batch_size,
            auth_endpoints,
            client_id,
            redirect_uri,
            issuer_state,
            wia_client,
            authorization_server,
        )
        .await
    }

    async fn start_pre_authorized_code_flow<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
    ) -> Result<Self::Issuance, WalletIssuanceError>
    where
        W: WiaClient,
    {
        let IssuanceDiscoveryParameters {
            offer_uri,
            selection,
            wia_client,
            wrpac_trust_anchors,
            wrprc_trust_anchors,
        } = common_parameters;

        let (credential_configurations, credential_identifier, issuer_endpoints, issuer_registration, batch_size, flow) =
            self.resolve_credential_offer_flow(offer_uri, selection, wrpac_trust_anchors, wrprc_trust_anchors)
                .await?;

        let CredentialOfferFlow::PreAuthorizedCode {
            pre_authorized_code,
            token_endpoint,
            authorization_server,
            challenge_endpoint,
        } = flow
        else {
            return Err(WalletIssuanceError::CredentialOfferNoPreAuthorizedCode);
        };

        self.create_issuance_session(
            pre_authorized_code,
            credential_configurations,
            credential_identifier,
            issuer_endpoints,
            issuer_registration,
            batch_size,
            token_endpoint,
            challenge_endpoint,
            wia_client,
            &authorization_server,
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
        challenge_endpoint: Option<Url>,
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
        oauth_metadata: IssuerAuthorizationServerMetadata,
    ) -> Result<Self, WalletIssuanceError> {
        let flow = match offer_grant {
            CredentialOfferGrant::AuthorizationCode { issuer_state } => {
                let authorization_server = oauth_metadata.oauth_metadata.issuer.clone();

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
                authorization_server: oauth_metadata.oauth_metadata.issuer,
                token_endpoint: oauth_metadata.oauth_metadata.token_endpoint,
                challenge_endpoint: oauth_metadata.client_attestation_metadata_extension.challenge_endpoint,
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
                    .oauth_metadata
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

                let authorization_server = oauth_metadata.oauth_metadata.issuer.clone();

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

impl<F, C> HttpIssuanceDiscovery<F, C>
where
    F: CrlFetcher,
    C: StatusListClient,
{
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
        wrprc_trust_anchors: &TrustAnchors,
    ) -> Result<(IssuerMetadata, IssuerAuthorizationServerMetadata, IssuerRegistration), WalletIssuanceError> {
        let issuer_metadata_jwt: UnverifiedJwt<SignedIssuerMetadataPayload, HeaderWithX5c> = self
            .http_client
            .get_jwt(IssuerMetadata::well_known_url(&credential_offer.credential_issuer))
            .await
            .map_err(WalletIssuanceError::CredentialIssuerMetadataHttp)?
            .parse()
            .map_err(WalletIssuanceError::JwtParse)?;

        let verified_issuer_metadata = issuer_metadata_jwt
            .into_verified_against_trust_anchors_with_crl(
                wrpac_trust_anchors,
                &self.crl_verifier,
                &TimeGenerator,
                None,
                DEFAULT_VALIDATION.to_owned(),
            )
            .await
            .map_err(WalletIssuanceError::CredentialIssuerMetadataVerify)?;
        let access_certificate = verified_issuer_metadata.header().x5c.first().clone();
        let issuer_metadata_payload = verified_issuer_metadata.into_payload();
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

        let registration_certificate = issuer_metadata
            .issuer_info
            .iter()
            .flatten()
            .filter_map(|info| match info {
                IssuerInfo::RegistrationCertificate { data } => Some(data),
                IssuerInfo::Other => None,
            })
            .at_most_one()
            .map_err(|_| RegistrationCertificateError::Multiple)
            .and_then(|certificate| certificate.ok_or(RegistrationCertificateError::Missing))
            .map_err(WalletIssuanceError::IssuerRegistrationCertificate)?;

        let validated = validate_registration_certificate(
            registration_certificate,
            &access_certificate,
            wrprc_trust_anchors,
            &self.wrprc_revocation_verifier,
            &TimeGenerator,
        )
        .await
        .map_err(WalletIssuanceError::IssuerRegistrationCertificate)?;

        let issuer_registration =
            IssuerRegistration::new(registration_certificate.clone(), access_certificate, &validated);

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

        let oauth_metadata =
            IssuerAuthorizationServerMetadata::fetch_well_known_json(&self.http_client, authorization_server)
                .await
                .map_err(WalletIssuanceError::OauthDiscovery)?;

        Ok((issuer_metadata, oauth_metadata, issuer_registration))
    }

    /// Parse or fetch the [`CredentialOffer`], fetch both the issuer and OAuth metadata and determine the flow type.
    async fn resolve_credential_offer_flow(
        &self,
        offer_uri: &Url,
        selection: &CredentialSelection,
        wrpac_trust_anchors: &TrustAnchors,
        wrprc_trust_anchors: &TrustAnchors,
    ) -> Result<
        (
            HashMap<CredentialConfigurationId, CredentialConfiguration>,
            IssuerIdentifier,
            IssuerEndpoints,
            IssuerRegistration,
            NonZeroU8,
            CredentialOfferFlow,
        ),
        WalletIssuanceError,
    > {
        let credential_offer = self.process_credential_offer(offer_uri).await?;

        let (issuer_metadata, oauth_metadata, issuer_registration) = self
            .fetch_metadata(&credential_offer, wrpac_trust_anchors, wrprc_trust_anchors)
            .await?;

        check_client_attestation_metadata(
            &oauth_metadata.oauth_metadata,
            &oauth_metadata.client_attestation_metadata_extension,
        )
        .map_err(WalletIssuanceError::ClientAttestationMetadata)?;

        // Limit credential copy count to a sane maximum, even if the issuer indicates it can provide more copies.
        let batch_size = std::cmp::min(issuer_metadata.batch_size(), BATCH_SIZE_MAX.into())
            .try_into()
            .expect("IssuerMetadata batch size is capped to BATCH_SIZE_MAX, which is u8");

        let IssuerMetadata {
            credential_issuer,
            endpoints: issuer_endpoints,
            mut credential_configurations_supported,
            ..
        } = issuer_metadata;

        // Collect the indices of all Credential Configuration IDs that appear in the Credential Offer, but not in the
        // Issuer Metadata. If any are missing we can use these indices to collect the owned values for returning the
        // error.
        let (credential_configs, missing_ids): (Vec<_>, Vec<_>) = credential_offer
            .credential_configuration_ids
            .into_iter()
            .enumerate()
            .partition_map(|(_index, id)| match credential_configurations_supported.remove(&id) {
                Some(config) => Either::Left((id, config)),
                None => Either::Right(id),
            });

        if !missing_ids.is_empty() {
            return Err(WalletIssuanceError::MissingCredentialConfigId(missing_ids));
        }

        // According to HAIP, if the issuer requires key binding for any of its credential configurations, it MUST also
        // offer a nonce endpoint. As the wallet, we interpret this a bit more loosely and reject issuance whenever any
        // of the credential configurations offered require key binding, as the metadata may contain other
        // configurations that do not concern this particular issuance session.
        // See: https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.1-5
        if issuer_endpoints.nonce_endpoint.is_none()
            && credential_configs
                .iter()
                .any(|(_id, config)| config.cryptographic_binding.is_some())
        {
            return Err(WalletIssuanceError::NoNonceEndpoint);
        }

        // Now that we have all the Credential Configurations that were part of the Credential Offer extracted from the
        // Issuer Metadata, select only those whose `CredentialKind` was requested.
        let selected_credential_configs = match selection {
            CredentialSelection::All => credential_configs,
            CredentialSelection::ByCredentialKind(credential_kinds) => {
                let (selected_credential_configs, other_credential_kinds): (Vec<_>, Vec<_>) =
                    credential_configs.into_iter().partition_map(|(id, config)| {
                        let credential_kind = config.format.credential_kind();

                        if credential_kind
                            .as_ref()
                            .is_some_and(|credential_kind| credential_kinds.contains(credential_kind))
                        {
                            Either::Left((id, config))
                        } else {
                            Either::Right(credential_kind)
                        }
                    });

                // It is an error if none of the requested `CredentialKind`s match the configurations on offer.
                if selected_credential_configs.is_empty() {
                    return Err(WalletIssuanceError::CredentialKindsNotOffered {
                        requested: credential_kinds.clone(),
                        offered: other_credential_kinds.into_iter().flatten().collect(),
                    });
                }

                selected_credential_configs
            }
        };

        let flow = CredentialOfferFlow::try_from_offer_grant(credential_offer.grant, oauth_metadata)?;

        Ok((
            selected_credential_configs.into_iter().collect(),
            credential_issuer,
            issuer_endpoints,
            issuer_registration,
            batch_size,
            flow,
        ))
    }

    #[expect(clippy::too_many_arguments, reason = "internal helper method")]
    async fn create_issuance_session(
        &self,
        pre_authorized_code: AuthorizationCode,
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        credential_issuer: IssuerIdentifier,
        issuer_endpoints: IssuerEndpoints,
        issuer_registration: IssuerRegistration,
        batch_size: NonZeroU8,
        token_endpoint: Url,
        challenge_endpoint: Option<Url>,
        wia_client: &impl WiaClient,
        authorization_server: &IssuerIdentifier,
    ) -> Result<HttpIssuanceSession, WalletIssuanceError> {
        let message_client = HttpVcMessageClient::new(self.http_client.clone());

        let token_request = VciTokenRequest::new_pre_authorized(pre_authorized_code);

        // In the pre-authorized code flow, no PAR request was sent whose response might have included a
        // challenge for Attestation-Based Client Authentication. So we can either use the challenge_endpoint,
        // if the issuer has one, or the issuer does not use WIA PoP challenges so we don't use one.
        let client_auth_challenge = ClientAttestationChallengeMechanism::new_pre_authorized(challenge_endpoint);

        HttpIssuanceSession::create(
            message_client,
            credential_configurations,
            credential_issuer,
            issuer_endpoints,
            issuer_registration,
            batch_size,
            token_endpoint,
            client_auth_challenge,
            token_request,
            wia_client,
            authorization_server,
        )
        .await
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
    use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
    use attestation_data::registration_certificate::RegistrationCertificateValidationError;
    use attestation_data::registration_certificate::mock::MockRegistrationCertificate;
    use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use attestation_data::registration_certificate::mock::issuer_registration_certificate_payload;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::DateTime;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::crl::CertificateCrlVerificationError;
    use crypto::x509::crl::CertificateCrlVerifier;
    use crypto::x509::crl::mock::MockCrlFetcher;
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
    use oauth::issuer_identifier::IssuerIdentifier;
    use rstest::rstest;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use serde_json::json;
    use token_status_list::status_list::StatusType;
    use token_status_list::verification::client::mock::MockStatusListClient;
    use url::Url;
    use utils::date_time_seconds::DateTimeSeconds;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::payload::wia::WIA_CLIENT_AUTH_METHOD;
    use wscd::wia::mock::MockWiaClient;

    use super::CredentialSelection;
    use super::HttpIssuanceDiscovery;
    use super::IssuanceDiscovery;
    use super::IssuanceDiscoveryParameters;
    use crate::authorization_details::AuthorizationDetails;
    use crate::client_auth::ClientAttestationMetadataError;
    use crate::credential_offer::CredentialOffer;
    use crate::credential_offer::CredentialOfferContainer;
    use crate::credential_offer::GrantPreAuthorizedCode;
    use crate::credential_offer::Grants;
    use crate::credential_offer::PreAuthTransactionCode;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::IssuerInfo;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::issuer_metadata::JoinCredentialConfigurationId;
    use crate::metadata::issuer_metadata::SignedIssuerMetadataPayload;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::preview::CredentialPreviewResponse;
    use crate::registration_certificate::RegistrationCertificateError;
    use crate::token::CredentialPreview;
    use crate::token::VciTokenResponse;
    use crate::wallet_issuance::AuthorizationSession;
    use crate::wallet_issuance::IssuanceFlow;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::authorization::HttpAuthorizationSession;
    use crate::wallet_issuance::discovery::BATCH_SIZE_MAX;
    use crate::wallet_issuance::issuance_session::HttpIssuanceSession;

    static CONFIG_ID_MDOC: LazyLock<CredentialConfigurationId> = LazyLock::new(|| "pid_mdoc".to_string().into());
    static CONFIG_ID_SD_JWT: LazyLock<CredentialConfigurationId> = LazyLock::new(|| "pid_sd_jwt".to_string().into());
    const CREDENTIAL_ID: &str = "credential_id";

    const DEFAULT_GRANT_TYPES_SUPPORTED: &[&str] = &[
        "authorization_code",
        "urn:ietf:params:oauth:grant-type:pre-authorized_code",
    ];
    static REDIRECT_URI: LazyLock<Url> = LazyLock::new(|| "https://wallet.example.com/callback".parse().unwrap());
    const AUTHORIZATION_ENDPOINT: &str = "https://auth.example.com/authorize";

    fn mock_discovery() -> HttpIssuanceDiscovery<MockCrlFetcher, MockStatusListClient> {
        HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            CertificateCrlVerifier::<MockCrlFetcher>::default(),
            MockStatusListClient::default(),
        )
    }

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
        options: IssuerMetadataOptions<'_>,
        to_signed_metadata: impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a>,
    ) -> (
        IssuerIdentifier,
        TrustAnchors,
        CertificateCrlVerifier<MockCrlFetcher>,
        MockRegistrationCertificate,
    ) {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let wrpac_keypair = if options.with_crl {
            ca.generate_wrpac_issuer_mock_with_crl().unwrap()
        } else {
            ca.generate_wrpac_issuer_mock().unwrap()
        };
        let crl_verifier = CertificateCrlVerifier::<MockCrlFetcher>::new_for_ca(&ca);

        let issuer_identifier = server.base_url().parse::<IssuerIdentifier>().unwrap();

        // Construct issuer metadata JSON.
        let mut issuer_metadata_json = json!({
            "credential_issuer": issuer_identifier.to_string(),
            "credential_endpoint": server.url("/issuance/credential"),
            "credential_preview_endpoint": server.url("/issuance/credential_preview"),
            "batch_credential_issuance": {
                "batch_size": 1000,
            },
            "credential_configurations_supported": {
                CONFIG_ID_MDOC.as_ref(): {
                    "format": "mso_mdoc",
                    "doctype": PID_ATTESTATION_TYPE,
                    "scope": "pid_mdoc_scope",
                    // Note that this is deliberately still present, as the wallet should ignore it for an mdoc.
                    "type_metadata_uri": issuer_identifier
                                            .as_issuer_url()
                                            .join_issuer_url("/issuance/type_metadata")
                                            .join_config_id(&CONFIG_ID_MDOC),
                    "credential_metadata": {
                        "display": [{ "name": "PID", "locale": "en" }],
                        "claims": [{
                            "path": ["family_name"],
                            "display": [{ "name": "Family name", "locale": "en" }],
                        }],
                    },
                },
                CONFIG_ID_SD_JWT.as_ref(): {
                    "format": "dc+sd-jwt",
                    "vct": PID_ATTESTATION_TYPE,
                    "scope": "pid_sd_jwt_scope",
                    "type_metadata_uri": issuer_identifier
                                            .as_issuer_url()
                                            .join_issuer_url("/issuance/type_metadata")
                                            .join_config_id(&CONFIG_ID_SD_JWT),
                }
            },
        });
        if options.requires_key_binding {
            let config = &mut issuer_metadata_json["credential_configurations_supported"][CONFIG_ID_MDOC.as_ref()];
            config["cryptographic_binding_methods_supported"] = json!(["jwk"]);
            config["proof_types_supported"] = json!({
                "jwt": { "proof_signing_alg_values_supported": ["ES256"] }
            });
        }
        if options.has_nonce_endpoint {
            issuer_metadata_json["nonce_endpoint"] = json!(server.url("/issuance/nonce"));
        }

        let mut payload = issuer_registration_certificate_payload(
            wrpac_keypair.certificate(),
            [Format::MsoMdoc, Format::SdJwt]
                .map(|format| CredentialKind::new(format, PID_ATTESTATION_TYPE.to_string())),
        );
        match options.registration_certificate {
            RegistrationCertificateScenario::WrongSubject => payload.0["sub"] = json!("another-issuer"),
            RegistrationCertificateScenario::Expired => {
                payload.0["iat"] = json!(12345678);
                payload.0["exp"] = json!(12345679);
            }
            RegistrationCertificateScenario::InvalidPayload => payload.0["id"] = json!(null),
            _ => {}
        }
        let authority = MockRegistrationCertificateAuthority::new_with_status(
            if matches!(
                options.registration_certificate,
                RegistrationCertificateScenario::Revoked
            ) {
                StatusType::Invalid
            } else {
                StatusType::Valid
            },
        );
        let certificate = if matches!(options.registration_certificate, RegistrationCertificateScenario::Cwt) {
            authority.sign_cwt(&payload)
        } else {
            authority.sign_jwt(&payload)
        };
        let registration_certificate = MockRegistrationCertificate {
            certificate,
            trust_anchors: authority.trust_anchors,
            status_list_client: if matches!(
                options.registration_certificate,
                RegistrationCertificateScenario::UntrustedStatusList
            ) {
                MockRegistrationCertificateAuthority::new().status_list_client
            } else {
                authority.status_list_client
            },
        };
        let mut issuer_metadata: IssuerMetadata = serde_json::from_value(issuer_metadata_json).unwrap();
        let issuer_info = IssuerInfo::RegistrationCertificate {
            data: RegistrationCertificateEnvelope::try_from(registration_certificate.certificate.as_slice()).unwrap(),
        };
        issuer_metadata.issuer_info = options
            .has_other_issuer_info
            .then_some(IssuerInfo::Other)
            .into_iter()
            .chain(std::iter::repeat_n(issuer_info, options.registration_certificate_count))
            .collect_vec()
            .try_into()
            .ok();
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
        if let Some(grant_types_supported) = options.grant_types_supported {
            oauth_metadata_json["grant_types_supported"] = json!(grant_types_supported);
        }
        if options.has_client_attestation_support {
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

        (
            issuer_identifier,
            TrustAnchors::from(&ca),
            crl_verifier,
            registration_certificate,
        )
    }

    #[derive(Debug, Clone, Copy, Default)]
    enum RegistrationCertificateScenario {
        #[default]
        Jwt,
        Cwt,
        WrongSubject,
        Expired,
        InvalidPayload,
        Revoked,
        UntrustedStatusList,
    }

    #[derive(Debug, Clone, Copy)]
    struct IssuerMetadataOptions<'a> {
        has_nonce_endpoint: bool,
        requires_key_binding: bool,
        grant_types_supported: Option<&'a [&'a str]>,
        has_client_attestation_support: bool,
        with_crl: bool,
        registration_certificate_count: usize,
        has_other_issuer_info: bool,
        registration_certificate: RegistrationCertificateScenario,
    }

    impl Default for IssuerMetadataOptions<'static> {
        fn default() -> Self {
            Self {
                has_nonce_endpoint: true,
                requires_key_binding: true,
                grant_types_supported: Some(DEFAULT_GRANT_TYPES_SUPPORTED),
                has_client_attestation_support: true,
                with_crl: true,
                registration_certificate_count: 1,
                has_other_issuer_info: false,
                registration_certificate: RegistrationCertificateScenario::default(),
            }
        }
    }

    /// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
    /// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
    async fn start_httpmock_issuer(
        metadata_options: IssuerMetadataOptions<'_>,
    ) -> (
        MockServer,
        IssuerIdentifier,
        TrustAnchors,
        TrustAnchors,
        CertificateCrlVerifier<MockCrlFetcher>,
        MockRegistrationCertificate,
    ) {
        let server = MockServer::start_async().await;

        // Create CA and issuer certificate for the credential preview.
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();

        // Create type metadata for the credential preview.
        let (_, _, type_metadata_documents) = TypeMetadataDocuments::from_single_example(
            TypeMetadata::example_with_claim_name(PID_ATTESTATION_TYPE, "family_name"),
        );

        let credential_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let preview = CredentialPreview {
            credential_id: CREDENTIAL_ID.to_string().into(),
            config_id: CONFIG_ID_MDOC.clone(),
            format: Format::MsoMdoc,
            credential_payload,
        };

        let preview_response = CredentialPreviewResponse {
            credential_previews: vec_nonempty![preview],
        };

        let token_response = VciTokenResponse::new_vci(
            "mock_access_token".to_string().into(),
            Some(AuthorizationDetails::from_credential_ids_and_identifiers(
                vec_nonempty![(LazyLock::force(&CONFIG_ID_MDOC), CREDENTIAL_ID.to_string().into())],
            )),
        );

        let (issuer_identifier, wrpac_trust_anchors, crl_verifier, registration_certificate) =
            httpmock_issuer_add_metadata(&server, metadata_options, default_signed_metadata()).await;

        server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(format!("/issuance/type_metadata/{}", CONFIG_ID_MDOC.as_ref()));
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
            crl_verifier,
            registration_certificate,
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

        let (
            server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions {
            grant_types_supported: has_grant_types_supported.then_some(DEFAULT_GRANT_TYPES_SUPPORTED),
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
            credential_configuration_ids: vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
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
        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );
        let flow = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
                        IssuanceDiscoveryParameters::new(
                            &offer_url,
                            &CredentialSelection::All,
                            &MockWiaClient::new(),
                            &wrpac_trust_anchors,
                            &registration_certificate.trust_anchors,
                        ),
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        REDIRECT_URI.clone(),
                    )
                    .await
                    .expect("starting authorization code issuance should succeed");

                // Staring issuance while expecting a Pre-Authorized Code flow results in an error.
                let error = discovery
                    .start_pre_authorized_code_flow(IssuanceDiscoveryParameters::new(
                        &offer_url,
                        &CredentialSelection::All,
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                        &registration_certificate.trust_anchors,
                    ))
                    .await
                    .expect_err("staring pre-authorized code issuance should fail");

                assert_matches!(error, WalletIssuanceError::CredentialOfferNoPreAuthorizedCode);

                // Resume one flow from persisted wallet state before exchanging the authorization code.
                let second_auth_session = HttpAuthorizationSession::restore(
                    HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
                    serde_json::from_value(serde_json::to_value(second_auth_session.persist()).unwrap()).unwrap(),
                );

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
                                .start_issuance(&received_redirect_uri, &MockWiaClient::new())
                                .await
                        }),
                )
                .await
                .unwrap()
            }
            (IssuanceDiscoveryScenario::PreAuthorizedCode, IssuanceFlow::PreAuthorizedCode { issuance_session }) => {
                // Start issuance again, this time directly expecting the Pre-Authorized Code flow.
                let second_issuance_session = discovery
                    .start_pre_authorized_code_flow(IssuanceDiscoveryParameters::new(
                        &offer_url,
                        &CredentialSelection::All,
                        &MockWiaClient::new(),
                        &wrpac_trust_anchors,
                        &registration_certificate.trust_anchors,
                    ))
                    .await
                    .expect("staring pre-authorized code issuance should succeed");

                // Staring issuance while expecting an Authorization Code flow results in an error.
                let error = discovery
                    .start_authorization_code_flow(
                        IssuanceDiscoveryParameters::new(
                            &offer_url,
                            &CredentialSelection::All,
                            &MockWiaClient::new(),
                            &wrpac_trust_anchors,
                            &registration_certificate.trust_anchors,
                        ),
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        REDIRECT_URI.clone(),
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
            let context = issuance_session.issuer_registration();
            assert_eq!(context.organization().display_name, "Mock issuer");
            assert_eq!(
                context.organization().description[0].translations[1].value,
                "Add digital documents to your wallet to share your details with other organizations."
            );
            assert_eq!(
                context.registration_certificate().to_vec().unwrap(),
                registration_certificate.certificate
            );
            let subject = context.access_certificate().to_distinguished_name().unwrap();
            assert_eq!(
                Some(&context.organization().identifier),
                subject.organization_identifier.as_ref()
            );

            // Check that the issuance session contains the expected credential preview.
            let Ok((preview, _metadata)) = issuance_session.previews_with_metadata().unwrap().exactly_one() else {
                panic!("issuance session should contain exactly one preview")
            };
            assert_eq!(preview.credential_payload.attestation_type, PID_ATTESTATION_TYPE);

            // Check that the batch size from the Issuer Metadata was capped.
            assert_eq!(issuance_session.batch_size(), BATCH_SIZE_MAX);
        }
    }

    #[rstest]
    #[case::missing(0, false)]
    #[case::other_only(0, true)]
    #[case::multiple(2, false)]
    #[case::single(1, false)]
    #[case::single_with_other(1, true)]
    #[tokio::test]
    async fn start_registration_certificate_selection(
        #[case] registration_certificate_count: usize,
        #[case] has_other_issuer_info: bool,
    ) {
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions {
            registration_certificate_count,
            has_other_issuer_info,
            ..IssuerMetadataOptions::default()
        })
        .await;
        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();
        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client,
        );
        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await;

        match registration_certificate_count {
            0 => assert_matches!(
                result,
                Err(WalletIssuanceError::IssuerRegistrationCertificate(
                    RegistrationCertificateError::Missing
                ))
            ),
            1 => assert_matches!(result, Ok(IssuanceFlow::PreAuthorizedCode { .. })),
            _ => assert_matches!(
                result,
                Err(WalletIssuanceError::IssuerRegistrationCertificate(
                    RegistrationCertificateError::Multiple
                ))
            ),
        }
    }

    #[tokio::test]
    async fn start_missing_query() {
        let discovery = mock_discovery();
        let offer_url = Url::parse("openid-credential-offer://").unwrap();

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &TrustAnchors::empty(),
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::MissingCredentialOfferQuery));
    }

    #[tokio::test]
    async fn start_deserialization_error() {
        let discovery = mock_discovery();
        let offer_url = Url::parse("openid-credential-offer://?credential_offer=invalid_json").unwrap();

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &TrustAnchors::empty(),
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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

        let discovery = mock_discovery();

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &TrustAnchors::empty(),
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
            "credential_configuration_ids": [CONFIG_ID_MDOC.as_ref()],
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

        let discovery = mock_discovery();

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &TrustAnchors::empty(),
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
        let (credential_issuer, wrpac_trust_anchors, crl_verifier, registration_certificate) =
            httpmock_issuer_add_metadata(
                &server,
                IssuerMetadataOptions {
                    grant_types_supported: Some(&["implicit"]),
                    ..IssuerMetadataOptions::default()
                },
                default_signed_metadata(),
            )
            .await;

        // Construct a Credential Offer that contains no grants.
        let credential_offer = CredentialOffer {
            credential_issuer,
            credential_configuration_ids: vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            grants: None,
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
            credential_configuration_ids: vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
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

        let discovery = mock_discovery();

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &TrustAnchors::empty(),
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await;

        assert_matches!(result, Err(WalletIssuanceError::CredentialOfferTxCodeUnsupported));
    }

    async fn start_check_metadata<'a>(
        to_signed_metadata: impl FnOnce(IssuerIdentifier, IssuerMetadata) -> SignedIssuerMetadataPayload<'a>,
        use_trust_anchors: bool,
        with_crl: bool,
    ) -> (
        IssuerIdentifier,
        Result<IssuanceFlow<HttpAuthorizationSession, HttpIssuanceSession>, WalletIssuanceError>,
    ) {
        let server = MockServer::start_async().await;

        // Setup simple metadata server
        let (credential_issuer, wrpac_trust_anchors, mock_crl_verifier, registration_certificate) =
            httpmock_issuer_add_metadata(
                &server,
                IssuerMetadataOptions {
                    has_nonce_endpoint: false,
                    requires_key_binding: false,
                    grant_types_supported: None,
                    with_crl,
                    ..IssuerMetadataOptions::default()
                },
                to_signed_metadata,
            )
            .await;

        // Construct a Credential Offer
        let credential_offer = CredentialOffer {
            credential_issuer: credential_issuer.clone(),
            credential_configuration_ids: vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            grants: None,
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        // Start discovery
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let trust_anchors = if use_trust_anchors {
            wrpac_trust_anchors
        } else {
            TrustAnchors::empty()
        };
        let result = HttpIssuanceDiscovery::new(
            http_client,
            mock_crl_verifier,
            registration_certificate.status_list_client,
        )
        .start(
            IssuanceDiscoveryParameters::new(
                &offer_url,
                &CredentialSelection::All,
                &MockWiaClient::new(),
                &trust_anchors,
                &registration_certificate.trust_anchors,
            ),
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.clone(),
        )
        .await;
        (credential_issuer, result)
    }

    #[tokio::test]
    async fn start_cwt_registration_certificate() {
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions {
            registration_certificate: RegistrationCertificateScenario::Cwt,
            ..IssuerMetadataOptions::default()
        })
        .await;
        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();
        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client,
        );
        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await;

        let IssuanceFlow::PreAuthorizedCode { issuance_session } = result.unwrap() else {
            panic!("expected pre-authorized flow");
        };
        let context = issuance_session.issuer_registration();
        assert_eq!(context.organization().display_name, "Mock issuer");
        assert_eq!(
            context.registration_certificate().to_vec().unwrap(),
            registration_certificate.certificate
        );
    }

    #[rstest]
    #[case::wrong_subject(RegistrationCertificateScenario::WrongSubject)]
    #[case::expired(RegistrationCertificateScenario::Expired)]
    #[case::invalid_payload(RegistrationCertificateScenario::InvalidPayload)]
    #[case::revoked(RegistrationCertificateScenario::Revoked)]
    #[case::untrusted_status_list(RegistrationCertificateScenario::UntrustedStatusList)]
    #[tokio::test]
    async fn start_invalid_registration_certificate(#[case] scenario: RegistrationCertificateScenario) {
        let server = MockServer::start_async().await;
        // No downstream request should be made when issuer authentication fails.
        let downstream = server
            .mock_async(|when, then| {
                when.path_excludes("/.well-known/openid-credential-issuer");
                then.status(500);
            })
            .await;
        let (credential_issuer, wrpac_trust_anchors, crl_verifier, registration_certificate) =
            httpmock_issuer_add_metadata(
                &server,
                IssuerMetadataOptions {
                    registration_certificate: scenario,
                    ..IssuerMetadataOptions::default()
                },
                default_signed_metadata(),
            )
            .await;
        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            credential_issuer,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();
        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client,
        );
        let error = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .unwrap_err();
        match scenario {
            RegistrationCertificateScenario::WrongSubject => {
                assert_matches!(
                    error,
                    WalletIssuanceError::IssuerRegistrationCertificate(RegistrationCertificateError::BindingAndTime(
                        RegistrationCertificateValidationError::SubjectIdentifierMismatch {
                            field: "organizationIdentifier"
                        }
                    ))
                );
            }
            RegistrationCertificateScenario::Expired | RegistrationCertificateScenario::InvalidPayload => {
                assert_matches!(
                    error,
                    WalletIssuanceError::IssuerRegistrationCertificate(RegistrationCertificateError::Envelope(_))
                );
            }
            RegistrationCertificateScenario::Revoked => {
                assert_matches!(
                    error,
                    WalletIssuanceError::IssuerRegistrationCertificate(RegistrationCertificateError::Status(
                        RegistrationCertificateStatusValidationError::NotValid
                    ))
                );
            }
            RegistrationCertificateScenario::UntrustedStatusList => {
                assert_matches!(
                    error,
                    WalletIssuanceError::IssuerRegistrationCertificate(RegistrationCertificateError::Status(
                        RegistrationCertificateStatusValidationError::InvalidStatusListReference
                    ))
                );
            }
            _ => panic!("expected an invalid registration certificate scenario"),
        }
        downstream.assert_calls_async(0).await;
    }

    #[tokio::test]
    async fn start_untrusted_registration_certificate() {
        let server = MockServer::start_async().await;
        let downstream = server
            .mock_async(|when, then| {
                when.path_excludes("/.well-known/openid-credential-issuer");
                then.status(500);
            })
            .await;
        let (credential_issuer, wrpac_trust_anchors, crl_verifier, registration_certificate) =
            httpmock_issuer_add_metadata(&server, IssuerMetadataOptions::default(), default_signed_metadata()).await;
        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            credential_issuer,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();
        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client,
        );
        let error = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &TrustAnchors::empty(),
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .unwrap_err();

        assert_matches!(
            error,
            WalletIssuanceError::IssuerRegistrationCertificate(RegistrationCertificateError::Envelope(_))
        );
        downstream.assert_calls_async(0).await;
    }

    #[tokio::test]
    async fn start_untrusted_metadata() {
        let (_, result) = start_check_metadata(default_signed_metadata(), false, true).await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataVerify(
                JwtX5cVerifyError::CertificateValidation(_)
            ))
        );
    }

    #[tokio::test]
    async fn start_metadata_with_crl_fails_without_distribution_point() {
        let (_, result) = start_check_metadata(default_signed_metadata(), true, false).await;

        assert_matches!(
            result,
            Err(WalletIssuanceError::CredentialIssuerMetadataVerify(
                JwtX5cVerifyError::CertificateCrlValidation(CertificateCrlVerificationError::NoCrlDistributionPoint)
            ))
        );
    }

    #[tokio::test]
    async fn start_expired_metadata() {
        let (_, result) = start_check_metadata(custom_signed_metadata(None, None, true), true, true).await;

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
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        // Construct a Pre-Authorized Code Credential Offer with an unknown Authorization Server.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier.clone(),
            credential_configuration_ids: vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
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

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        // Construct a Pre-Authorized Code Credential Offer with Credential Configurations ID that are not in the Issuer
        // Metadata.
        let credential_offer = CredentialOffer {
            credential_issuer: issuer_identifier,
            credential_configuration_ids: vec_nonempty![
                "other_id".to_string().into(),
                CONFIG_ID_MDOC.clone(),
                "another_id".to_string().into()
            ]
            .into(),
            grants: Some(Grants::new_pre_authorized("fake_pre_auth_code".to_string().into())),
        };
        let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let result = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
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
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions {
            has_nonce_endpoint: false,
            ..IssuerMetadataOptions::default()
        })
        .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let error = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .expect_err("starting issuance should fail");

        assert_matches!(error, WalletIssuanceError::NoNonceEndpoint);

        // When key binding is not mandatory however, the nonce endpoint can be absent.
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions {
            has_nonce_endpoint: false,
            requires_key_binding: false,
            ..IssuerMetadataOptions::default()
        })
        .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );
        let _flow = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .expect("starting issuance should succeed");
    }

    #[tokio::test]
    async fn start_credential_kind_not_offered_error() {
        // Starting issuance when the caller requests credential kinds that are not offered should fail.
        let (
            _server,
            issuer_identifier,
            _issuer_trust_anchors,
            wrpac_trust_anchors,
            crl_verifier,
            registration_certificate,
        ) = start_httpmock_issuer(IssuerMetadataOptions::default()).await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_SD_JWT.clone(), CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let requested_kinds = HashSet::from([
            CredentialKind::new(Format::MsoMdoc, "unknown_doc_type".to_string()),
            CredentialKind::new(Format::SdJwt, "unknown_vct".to_string()),
        ]);
        let error = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::ByCredentialKind(requested_kinds.clone()),
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .expect_err("starting issuance should fail");

        let offered_kinds = vec![
            CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            CredentialKind::new(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string()),
        ];
        assert_matches!(
            error,
            WalletIssuanceError::CredentialKindsNotOffered { requested, offered }
                if requested == requested_kinds && offered == offered_kinds
        );

        // Requesting at least one credential kind that is offered should succeed.
        let requested_kinds = HashSet::from([CredentialKind::new(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string())]);
        let _flow = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::ByCredentialKind(requested_kinds),
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .expect("starting issuance should succeed");
    }

    #[tokio::test]
    async fn start_no_attestation_based_client_auth_support_error() {
        // Starting issuance when the Authorization Server metadata does not advertise support for
        // Attestation-Based Client Authentication should fail.
        let (_server, issuer_identifier, _trust_anchor, wrpac_trust_anchors, crl_verifier, registration_certificate) =
            start_httpmock_issuer(IssuerMetadataOptions {
                has_client_attestation_support: false,
                ..IssuerMetadataOptions::default()
            })
            .await;

        let offer_url = CredentialOfferContainer::new_offer(CredentialOffer::new_pre_authorized(
            issuer_identifier,
            vec_nonempty![CONFIG_ID_MDOC.clone()].into(),
            "fake_pre_auth_code".to_string().into(),
        ))
        .to_credential_offer_url();

        let discovery = HttpIssuanceDiscovery::new(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            crl_verifier,
            registration_certificate.status_list_client.clone(),
        );

        let error = discovery
            .start(
                IssuanceDiscoveryParameters::new(
                    &offer_url,
                    &CredentialSelection::All,
                    &MockWiaClient::new(),
                    &wrpac_trust_anchors,
                    &registration_certificate.trust_anchors,
                ),
                MOCK_WALLET_CLIENT_ID.to_string(),
                REDIRECT_URI.clone(),
            )
            .await
            .expect_err("starting issuance should fail");

        assert_matches!(
            error,
            WalletIssuanceError::ClientAttestationMetadata(
                ClientAttestationMetadataError::NoAttestationBasedClientAuthSupport
            )
        );
    }
}
