use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::identity;
use std::num::NonZeroU8;

use attestation_data::attributes::AttributesTraversalBehaviour;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use crypto::PublicKey;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use derive_more::Debug;
use futures::TryFutureExt;
use futures::future::try_join_all;
use futures::try_join;
use http_utils::reqwest::HttpClient;
use indexmap::IndexMap;
use itertools::Either;
use itertools::Itertools;
use jwt::nonce::Nonce;
use jwt::wia::WIA_HEADER_NAME;
use jwt::wia::WIA_POP_HEADER_NAME;
use jwt::wia::WiaDisclosure;
use mdoc::ATTR_RANDOM_LENGTH;
use mdoc::holder::Mdoc;
use mdoc::utils::serialization::TaggedBytes;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use reqwest::Method;
use reqwest::Response;
use reqwest::header::AUTHORIZATION;
use sd_jwt::error::DecoderError;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
use url::Url;
use utils::generator::TimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wscd::wscd::IssuanceResult;
use wscd::wscd::IssuanceWscd;
use wscd::wscd::WiaClient;

use super::IssuanceSession;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use super::credential::IssuedCredentialCopies;
use super::credential::MdocCopy;
use super::credential::SdJwtCopy;
use crate::authorization_details::IssuerAuthorizationDetails;
use crate::client_auth::ClientAttestationChallengeMechanism;
use crate::client_auth::fetch_client_auth_challenge;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestIdentifier;
use crate::credential::CredentialResponse;
use crate::credential::Credentials;
use crate::credential::MdocCredential;
use crate::credential::SdJwtCredential;
use crate::dpop::DPOP_HEADER_NAME;
use crate::dpop::DPOP_NONCE_HEADER_NAME;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::dpop::DpopNonce;
use crate::errors::CredentialErrorCode;
use crate::errors::CredentialPreviewErrorCode;
use crate::errors::RemoteErrorCode;
use crate::errors::RemoteErrorResponse;
use crate::errors::TokenErrorCode;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::nonce::response::NonceResponse;
use crate::preview::CredentialPreviewResponse;
use crate::scope::Scope;
use crate::token::AccessToken;
use crate::token::CredentialPreview;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;
use crate::wallet_issuance::AcceptIssuanceSelection;

#[derive(Debug)]
pub struct HttpIssuanceSession<H = HttpVcMessageClient> {
    message_client: H,
    session_state: IssuanceState,
}

/// Contract for sending OpenID4VCI protocol messages.
#[cfg_attr(test, mockall::automock)]
pub trait VcMessageClient {
    async fn request_token(
        &self,
        url: Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(TokenResponse, Option<DpopNonce>), WalletIssuanceError>;

    async fn request_challenge(&self, url: Url) -> Result<Nonce, WalletIssuanceError>;

    async fn request_credential_preview(
        &self,
        url: Url,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError>;

    async fn request_type_metadata(&self, url: Url) -> Result<TypeMetadataDocuments, WalletIssuanceError>;

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<DpopNonce>), WalletIssuanceError>;

    async fn request_credential(
        &self,
        url: Url,
        credential_request: &CredentialRequest,
        dpop_header: &Dpop,
        access_token: &AccessToken,
    ) -> Result<CredentialResponse, WalletIssuanceError>;

    async fn reject(&self, url: Url, dpop_header: &Dpop, access_token: &AccessToken)
    -> Result<(), WalletIssuanceError>;
}

#[derive(Debug)]
pub struct HttpVcMessageClient {
    http_client: HttpClient,
}

impl HttpVcMessageClient {
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    fn dpop_nonce(response: &Response) -> Result<Option<DpopNonce>, WalletIssuanceError> {
        let dpop_nonce = response
            .headers()
            .get(DPOP_NONCE_HEADER_NAME)
            .map(|header| -> Result<_, WalletIssuanceError> {
                let dpop_nonce = header
                    .to_str()
                    .map_err(WalletIssuanceError::DpopNonceHeader)?
                    .parse()
                    .map_err(WalletIssuanceError::DpopNonce)?;

                Ok(dpop_nonce)
            })
            .transpose()?;

        Ok(dpop_nonce)
    }

    fn dpop_auth_header(access_token: &AccessToken) -> String {
        format!("DPoP {}", access_token.as_ref())
    }
}

impl VcMessageClient for HttpVcMessageClient {
    async fn request_token(
        &self,
        url: Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(TokenResponse, Option<DpopNonce>), WalletIssuanceError> {
        self.http_client
            .post(url, |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header.to_string())
                    .header(WIA_HEADER_NAME, wia.wia().to_string())
                    .header(WIA_POP_HEADER_NAME, wia.wia_pop().to_string())
                    .form(token_request)
            })
            .map_err(WalletIssuanceError::TokenRequestHttp)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();

                if status.is_client_error() || status.is_server_error() {
                    let error = response
                        .json::<RemoteErrorResponse<TokenErrorCode>>()
                        .await
                        .map_err(WalletIssuanceError::TokenRequestHttp)?;

                    Err(WalletIssuanceError::TokenRequest(Box::new(error)))
                } else {
                    let dpop_nonce = Self::dpop_nonce(&response)?;
                    let deserialized = response
                        .json::<TokenResponse>()
                        .await
                        .map_err(WalletIssuanceError::TokenRequestHttp)?;

                    Ok((deserialized, dpop_nonce))
                }
            })
            .await
    }

    async fn request_challenge(&self, challenge_endpoint: Url) -> Result<Nonce, WalletIssuanceError> {
        fetch_client_auth_challenge(&self.http_client, challenge_endpoint)
            .await
            .map_err(WalletIssuanceError::ClientAttestationChallenge)
    }

    async fn request_credential_preview(
        &self,
        url: Url,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError> {
        self.http_client
            .post(url, |builder| builder.bearer_auth(access_token.as_ref()))
            .map_err(WalletIssuanceError::CredentialPreviewHttp)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();

                if status.is_client_error() || status.is_server_error() {
                    let error = response
                        .json::<RemoteErrorResponse<CredentialPreviewErrorCode>>()
                        .await
                        .map_err(WalletIssuanceError::CredentialPreviewHttp)?;

                    Err(WalletIssuanceError::CredentialPreview(Box::new(error)))
                } else {
                    let response = response
                        .json()
                        .await
                        .map_err(WalletIssuanceError::CredentialPreviewHttp)?;

                    Ok(response)
                }
            })
            .await
    }

    async fn request_type_metadata(&self, url: Url) -> Result<TypeMetadataDocuments, WalletIssuanceError> {
        self.http_client
            .get_json(url)
            .await
            .map_err(WalletIssuanceError::TypeMetadataHttp)
    }

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<DpopNonce>), WalletIssuanceError> {
        let response = self
            .http_client
            .post(url, identity)
            .await
            .map_err(WalletIssuanceError::NonceHttp)?
            .error_for_status()
            .map_err(WalletIssuanceError::NonceHttp)?;

        let dpop_nonce = Self::dpop_nonce(&response)?;
        let nonce_response = response.json().await.map_err(WalletIssuanceError::NonceHttp)?;

        Ok((nonce_response, dpop_nonce))
    }

    async fn request_credential(
        &self,
        url: Url,
        credential_request: &CredentialRequest,
        dpop_header: &Dpop,
        access_token: &AccessToken,
    ) -> Result<CredentialResponse, WalletIssuanceError> {
        self.http_client
            .post(url, |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header.to_string())
                    .header(AUTHORIZATION, Self::dpop_auth_header(access_token))
                    .json(credential_request)
            })
            .map_err(WalletIssuanceError::CredentialRequestHttp)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();

                if status.is_client_error() || status.is_server_error() {
                    let error = response
                        .json::<RemoteErrorResponse<CredentialErrorCode>>()
                        .await
                        .map_err(WalletIssuanceError::CredentialRequestHttp)?;

                    Err(WalletIssuanceError::CredentialRequest(Box::new(error)))
                } else {
                    let response = response
                        .json()
                        .await
                        .map_err(WalletIssuanceError::CredentialRequestHttp)?;

                    Ok(response)
                }
            })
            .await
    }

    async fn reject(
        &self,
        url: Url,
        dpop_header: &Dpop,
        access_token: &AccessToken,
    ) -> Result<(), WalletIssuanceError> {
        self.http_client
            .delete(url, |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header.to_string())
                    .header(AUTHORIZATION, Self::dpop_auth_header(access_token))
            })
            .map_err(WalletIssuanceError::CredentialRejectionHttp)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();

                if status.is_client_error() || status.is_server_error() {
                    let error = response
                        .json::<RemoteErrorResponse<CredentialErrorCode>>()
                        .await
                        .map_err(WalletIssuanceError::CredentialRejectionHttp)?;

                    Err(WalletIssuanceError::CredentialRejection(Box::new(error)))
                } else {
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
enum OfferedCredentialConfigs {
    WithoutIdentifiers(HashMap<CredentialConfigurationId, CredentialConfiguration>),
    WithIdentifiers(HashMap<CredentialConfigurationId, (CredentialConfiguration, HashSet<String>)>),
}

#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    batch_size: NonZeroU8,
    type_metadata: HashMap<String, IssuanceTypeMetadata>,
    offered_credentials: OfferedCredentials,
    issuer_registration: IssuerRegistration,
    #[debug(skip)]
    dpop_signing_key: SigningKey,
    dpop_nonce: Option<DpopNonce>,
}

#[derive(Debug)]
struct IssuanceTypeMetadata {
    normalized_metadata: NormalizedTypeMetadata,
    raw_metadata: SortedTypeMetadataDocuments,
}

/// Internal state of credential previews offered by the issuer, indexed either by Credential Identifier or Credential
/// Configuration Identifier, depending on whether the Token Response contained `authorization_details`. Note that this
/// maintains the order as received from the Credential Preview endpoint.
#[derive(Debug)]
enum OfferedCredentials {
    CredentialIds(IndexMap<String, CredentialPreview>),
    CredentialConfigurationIds(IndexMap<CredentialConfigurationId, CredentialPreview>),
}

impl OfferedCredentials {
    pub fn credential_count(&self) -> usize {
        match self {
            OfferedCredentials::CredentialIds(previews_by_credential_id) => previews_by_credential_id.len(),
            OfferedCredentials::CredentialConfigurationIds(previews_by_config_id) => previews_by_config_id.len(),
        }
    }

    pub fn credential_previews(&self) -> impl Iterator<Item = &CredentialPreview> {
        match self {
            Self::CredentialIds(previews_by_credential_id) => Either::Left(previews_by_credential_id.values()),
            Self::CredentialConfigurationIds(previews_by_config_id) => Either::Right(previews_by_config_id.values()),
        }
    }

    pub fn select_identifiers_and_previews(
        &self,
        selection: &AcceptIssuanceSelection,
    ) -> impl Iterator<Item = (CredentialRequestIdentifier, &CredentialPreview)> {
        // Iterate over all of the offered credentials, creating the appropriate `CredentialRequestIdentifier` value.
        let all_credentials = match self {
            Self::CredentialIds(previews_by_credential_id) => {
                Either::Left(previews_by_credential_id.iter().map(|(credential_id, preview)| {
                    (
                        CredentialRequestIdentifier::CredentialIdentifier(credential_id.clone()),
                        preview,
                    )
                }))
            }
            Self::CredentialConfigurationIds(previews_by_config_id) => {
                Either::Right(previews_by_config_id.iter().map(|(config_id, preview)| {
                    (
                        CredentialRequestIdentifier::CredentialConfigurationId(config_id.clone()),
                        preview,
                    )
                }))
            }
        };

        // If the caller specified indices of credentials to be accepted, select only those for fetching.
        match selection {
            AcceptIssuanceSelection::All => Either::Left(all_credentials),
            AcceptIssuanceSelection::PreviewIndices(indices) => Either::Right(
                all_credentials
                    .enumerate()
                    .filter(|(index, _credential)| indices.contains(index))
                    .map(|(_index, credential)| credential),
            ),
        }
    }
}

impl IssuanceState {
    fn dpop_header(&self, url: Url, method: &Method, dpop_nonce: Option<DpopNonce>) -> Result<Dpop, DpopError> {
        let dpop_header = Dpop::new(
            &self.dpop_signing_key,
            url,
            method,
            Some(&self.access_token),
            dpop_nonce,
        )?;

        Ok(dpop_header)
    }
}

/// Detects if an issuance error that occurred during a token request is a PreAuthorizedCodeExpired error.
///
/// In the pre-authorized-code flow, an `invalid_grant` response at the token endpoint can only mean the code is no
/// longer valid: the session is missing (cleaned up), expired or already used. No PKCE / client_id / scope /
/// redirect_uri check that also yields `invalid_grant` applies to this grant type, so the translation is unambiguous
/// and lets the wallet render a dedicated "QR code no longer valid" screen (without the issuer having to return a
/// non-standard, non-spec-compliant error code).
///
/// The authorization-code flow is deliberately left untranslated: there `invalid_grant` is *also* returned for
/// PKCE verification, client_id mismatch, scope and redirect_uri failures, so it is not a reliable "code no longer
/// valid" signal. And the genuine "no longer valid" case — the session expiring or being consumed between the
/// authorization callback and the subsequent token request — is practically unreachable in the current implementation.
/// So the generic error handling is used.
fn map_pre_authorized_token_error(error: WalletIssuanceError, token_request: &TokenRequest) -> WalletIssuanceError {
    let is_pre_authorized = matches!(
        token_request.grant_type,
        TokenRequestGrantType::PreAuthorizedCode { .. }
    );

    match &error {
        WalletIssuanceError::TokenRequest(response)
            if is_pre_authorized && response.error == RemoteErrorCode::Known(TokenErrorCode::InvalidGrant) =>
        {
            WalletIssuanceError::PreAuthorizedCodeExpired
        }
        _ => error,
    }
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    #[expect(clippy::too_many_arguments, reason = "constructor method")]
    pub(crate) async fn create(
        message_client: H,
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        credential_issuer: IssuerIdentifier,
        issuer_endpoints: IssuerEndpoints,
        batch_size: NonZeroU8,
        token_endpoint: Url,
        client_auth_challenge: ClientAttestationChallengeMechanism,
        token_request: TokenRequest,
        wia_client: &impl WiaClient,
        auth_server_identifier: &IssuerIdentifier,
        trust_anchors: &TrustAnchors,
    ) -> Result<Self, WalletIssuanceError> {
        let credential_preview_endpoint = issuer_endpoints
            .credential_preview_endpoint
            .as_ref()
            .ok_or(WalletIssuanceError::NoCredentialPreviewEndpoint)?; // TODO (PVW-5559): skip preview when no credential preview endpoint

        let dpop_signing_key = SigningKey::generate();
        let dpop_header = Dpop::new(&dpop_signing_key, token_endpoint.clone(), &Method::POST, None, None)?;

        let challenge = match client_auth_challenge {
            ClientAttestationChallengeMechanism::None => None,
            ClientAttestationChallengeMechanism::Header(challenge) => Some(challenge),
            ClientAttestationChallengeMechanism::ChallengeEndpoint(url) => {
                Some(message_client.request_challenge(url).await?)
            }
        };

        let wia = wia_client
            .issue_wia(auth_server_identifier.to_string(), challenge)
            .await
            .map_err(|e| WalletIssuanceError::WiaIssuance(e.into()))?;

        let (token_response, dpop_nonce) = message_client
            .request_token(token_endpoint, &token_request, &dpop_header, &wia)
            .await
            .map_err(|error| map_pre_authorized_token_error(error, &token_request))?;

        let offered_credential_configs = Self::filter_offered_credential_configs(
            credential_configurations,
            token_response.scope.as_ref(),
            token_response.authorization_details,
        )?;

        // Request preview and fetch type metadata
        let (type_metadata, credential_previews) = try_join!(
            Self::fetch_type_metadata(&offered_credential_configs, &credential_issuer, &message_client),
            Self::request_previews(
                credential_preview_endpoint.as_url().clone(),
                &token_response.access_token,
                trust_anchors,
                &message_client
            )
        )?;

        let issuer_registration = credential_previews
            .iter()
            .map(|preview| preview.issuer_registration())
            .collect::<Result<Vec<_>, _>>()
            .map_err(WalletIssuanceError::PreviewIssuerRegistration)?
            .into_iter()
            // Use `dedup()` instead of `unique()`, as `IssuerRegistration` does not implement Hash. The end result is
            // the same when followed by `.exactly_one()`.
            .dedup()
            // Note that this iterator resulting in 0 values will never happen because `credential_previews` is
            // non-empty, so this error only occurs when there are multiple `IssuerRegistration` values.
            .exactly_one()
            .map_err(|_| WalletIssuanceError::DifferentIssuers)?;

        let offered_credentials =
            Self::match_preview_against_offered_credentials(credential_previews, offered_credential_configs)?;

        let session_state = IssuanceState {
            access_token: token_response.access_token,
            credential_issuer,
            issuer_endpoints,
            batch_size,
            offered_credentials,
            type_metadata,
            issuer_registration,
            dpop_signing_key,
            dpop_nonce,
        };

        let issuance_client = Self {
            message_client,
            session_state,
        };

        Ok(issuance_client)
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the fields received in
    /// the Token Response. Returns errors if any of the values in these fields is unrecognized.
    fn filter_offered_credential_configs(
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        scope: Option<&HashSet<Scope>>,
        authorization_details: Option<IssuerAuthorizationDetails>,
    ) -> Result<OfferedCredentialConfigs, WalletIssuanceError> {
        match (scope, authorization_details) {
            // If the Token Response contained `authorization_details`, use that and ignore any `scope` values. Returns
            // an error if any Credential Configuration ID was not present in the Credential Offer.
            (_, Some(authorization_details)) => {
                Self::filter_credential_configs_authorization_details(credential_configurations, authorization_details)
            }
            // If the Token Response contained `scope` values, select only those Credential Configurations that have
            // this scope. Returns an error if no scope values were provided or if any of the scope values do not refer
            // to Credential Configurations present in the Credential Offer.
            (Some(scope), None) => Self::filter_credential_configs_scope(credential_configurations, scope),
            // If neither the `authorization_details` nor the `scope` field was present in the Token Response, it means
            // that the issuer offers all of the Credential Configurations from the Credential Offer.
            (None, None) => Ok(OfferedCredentialConfigs::WithoutIdentifiers(credential_configurations)),
        }
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the
    /// `authorization_details` field.
    fn filter_credential_configs_authorization_details(
        mut credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        authorization_details: IssuerAuthorizationDetails,
    ) -> Result<OfferedCredentialConfigs, WalletIssuanceError> {
        let (offered_configs, unknown_config_ids): (HashMap<_, _>, Vec<_>) = authorization_details
            .into_credential_ids_and_identifiers()
            .into_iter()
            .partition_map(
                |(config_id, identifiers)| match credential_configurations.remove(&config_id) {
                    Some(config) => Either::Left((config_id, (config, identifiers.into_iter().collect()))),
                    None => Either::Right(config_id),
                },
            );

        if !unknown_config_ids.is_empty() {
            return Err(WalletIssuanceError::TokenResponseUnknownCredentialConfigIds(
                unknown_config_ids,
            ));
        }

        Ok(OfferedCredentialConfigs::WithIdentifiers(offered_configs))
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the `scope` field.
    fn filter_credential_configs_scope(
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        scope: &HashSet<Scope>,
    ) -> Result<OfferedCredentialConfigs, WalletIssuanceError> {
        if scope.is_empty() {
            return Err(WalletIssuanceError::TokenResponseEmptyScope);
        }

        let config_scopes = credential_configurations
            .values()
            .flat_map(|config| config.scope.as_ref())
            .collect::<HashSet<_>>();

        let unknown_scope = scope
            .iter()
            .filter(|scope| !config_scopes.contains(scope))
            .cloned()
            .collect_vec();

        if !unknown_scope.is_empty() {
            return Err(WalletIssuanceError::TokenResponseUnknownScope(unknown_scope));
        }

        let offered_configs = credential_configurations
            .into_iter()
            .filter(|(_config_id, config)| {
                config
                    .scope
                    .as_ref()
                    .is_some_and(|config_scope| scope.contains(config_scope))
            })
            .collect();

        Ok(OfferedCredentialConfigs::WithoutIdentifiers(offered_configs))
    }

    async fn request_previews(
        preview_endpoint: Url,
        access_token: &AccessToken,
        trust_anchors: &TrustAnchors,
        message_client: &H,
    ) -> Result<VecNonEmpty<CredentialPreview>, WalletIssuanceError> {
        let CredentialPreviewResponse { credential_previews } = message_client
            .request_credential_preview(preview_endpoint, access_token)
            .await?;

        // Verify the preview issuer certificates against the trust anchors.
        for preview in &credential_previews {
            preview
                .verify(trust_anchors)
                .map_err(WalletIssuanceError::CredentialPreviewVerification)?;
        }

        Ok(credential_previews)
    }

    /// Fetch SD-JWT VC Type Metadata for every Credential Configuration. This returns the resulting Type Metadata per
    /// attestation type, as each of these could occur in multiple Credential Configurations.
    async fn fetch_type_metadata(
        offered_credential_configs: &OfferedCredentialConfigs,
        credential_issuer: &IssuerIdentifier,
        message_client: &H,
    ) -> Result<HashMap<String, IssuanceTypeMetadata>, WalletIssuanceError> {
        // Get the metadata URI and attestation_type for each credential configuration, while collecting any Credential
        // Configuration IDs for which no type metadata URI is given.
        let (configs_data, missing_uri_config_ids): (Vec<_>, Vec<_>) = match offered_credential_configs {
            OfferedCredentialConfigs::WithoutIdentifiers(configs) => Either::Left(configs.iter()),
            OfferedCredentialConfigs::WithIdentifiers(configs) => Either::Right(
                configs
                    .iter()
                    .map(|(config_id, (config, _identifiers))| (config_id, config)),
            ),
        }
        .partition_map(|(config_id, config)| {
            match config.type_metadata_uri.as_ref() {
                Some(uri) => {
                    let attestation_type = config
                        .format
                        .attestation_type()
                        // TODO (PVW-6161): Handle unsupported formats earlier and more consistently.
                        .expect("unsupported format");

                    Either::Left((uri, attestation_type))
                }
                None => Either::Right(config_id.clone()),
            }
        });

        // TODO (PVW-5547): Use Credential Metadata from Issuer Metadata if type metadata URI is not present.
        if !missing_uri_config_ids.is_empty() {
            return Err(WalletIssuanceError::TypeMetadataUriMissing(missing_uri_config_ids));
        }

        // Transform this to all unique type metadata URIs, along with all configuration IDs and attestation types per
        // URI.
        let attestation_types_per_uri = configs_data.into_iter().into_group_map();

        // Check that all URIs have the same scheme and host as the Issuer Identifier, as is required by our profile.
        let mismatched_uris = attestation_types_per_uri
            .keys()
            .filter(|uri| !uri.has_same_scheme_and_host(credential_issuer.as_issuer_url()))
            .copied()
            .cloned()
            .collect_vec();

        if !mismatched_uris.is_empty() {
            return Err(WalletIssuanceError::TypeMetadataHostMismatch(
                Box::new(credential_issuer.clone()),
                Box::new(mismatched_uris),
            ));
        }

        // Make sure there is only one distinct attestation type per URI, while retaining the config IDs.
        let (attestation_types_and_uris, multi_attestation_type_uris): (Vec<_>, Vec<_>) = attestation_types_per_uri
            .into_iter()
            .partition_map(
                |(uri, attestation_types)| match attestation_types.into_iter().unique().exactly_one() {
                    Ok(attestation_type) => Either::Left((attestation_type, uri)),
                    Err(attestation_types_iter) => {
                        let attestation_types = attestation_types_iter.map(str::to_string).collect_vec();

                        Either::Right((uri.clone(), attestation_types))
                    }
                },
            );

        if !multi_attestation_type_uris.is_empty() {
            return Err(WalletIssuanceError::TypeMetadataUriMultipleAttestationTypes(Box::new(
                multi_attestation_type_uris,
            )));
        }

        // Fetch type metadata documents from URIs, then normalize the chain of documents.
        let metadata_per_attestation_type = try_join_all(attestation_types_and_uris.into_iter().map(
            async |(attestation_type, uri)| -> Result<_, WalletIssuanceError> {
                let documents = message_client.request_type_metadata(uri.as_url().clone()).await?;

                let (normalized_metadata, raw_metadata) = documents
                    .into_normalized(attestation_type)
                    .map_err(WalletIssuanceError::TypeMetadataVerification)?;

                let metadata = IssuanceTypeMetadata {
                    normalized_metadata,
                    raw_metadata,
                };

                Ok((attestation_type.to_string(), metadata))
            },
        ))
        .await?
        .into_iter()
        .collect();

        Ok(metadata_per_attestation_type)
    }

    /// Check that the `CredentialPreview`s exactly match the credentials that were offered by the issuer. This throws
    /// an error when any previews are missing or when excess previews are received.
    fn match_preview_against_offered_credentials(
        credential_previews: VecNonEmpty<CredentialPreview>,
        offered_credential_configs: OfferedCredentialConfigs,
    ) -> Result<OfferedCredentials, WalletIssuanceError> {
        let (offered_credentials, excess_identifiers): (_, Vec<_>) = match offered_credential_configs {
            // If the offered credential configurations did not contain credential identifiers because the issuer did
            // not send `authorization_details`, match every preview against its `config_id` value only.
            OfferedCredentialConfigs::WithoutIdentifiers(mut configs) => {
                let (previews_by_config_id, excess_identifiers) =
                    credential_previews.into_iter().partition_map(|preview| {
                        match configs.remove_entry(&preview.config_id) {
                            Some((config_id, _)) => Either::Left((config_id, preview)),
                            None => Either::Right((preview.config_id, preview.credential_id)),
                        }
                    });

                // If there are any offered credential configurations remaining, the preview did not contain everything
                // that was offered.
                if !configs.is_empty() {
                    let missing = configs.into_keys().map(|config_id| (config_id, None)).collect();

                    return Err(WalletIssuanceError::PreviewMissingCredentials(missing));
                }

                (
                    OfferedCredentials::CredentialConfigurationIds(previews_by_config_id),
                    excess_identifiers,
                )
            }
            // If the issuer did send `authorization_details`, match every preview exactly against both its `config_id`
            // and `credential_id` values.
            OfferedCredentialConfigs::WithIdentifiers(mut configs) => {
                let (previews_by_credential_id, excess_identifiers) =
                    credential_previews.into_iter().partition_map(|preview| {
                        match configs
                            .get_mut(&preview.config_id)
                            .and_then(|(_config, credential_ids)| credential_ids.take(&preview.credential_id))
                        {
                            Some(credential_id) => Either::Left((credential_id, preview)),
                            None => Either::Right((preview.config_id, preview.credential_id)),
                        }
                    });

                // If there are any offered credential identifiers remaining, the preview did not contain everything
                // that was offered.
                if configs
                    .values()
                    .any(|(_config, credential_ids)| !credential_ids.is_empty())
                {
                    let missing = configs
                        .into_iter()
                        .flat_map(|(config_id, (_config, credential_ids))| {
                            let credential_id_count = credential_ids.len();
                            std::iter::repeat_n(config_id, credential_id_count)
                                .zip(credential_ids)
                                .map(|(config_id, credential_id)| (config_id, Some(credential_id)))
                        })
                        .collect();

                    return Err(WalletIssuanceError::PreviewMissingCredentials(missing));
                }

                (
                    OfferedCredentials::CredentialIds(previews_by_credential_id),
                    excess_identifiers,
                )
            }
        };

        // If any of the previews could not be resolved against what was offered, report this as an error.
        if !excess_identifiers.is_empty() {
            return Err(WalletIssuanceError::PreviewExcessCredentials(excess_identifiers));
        }

        Ok(offered_credentials)
    }

    async fn fetch_credential<W>(
        &self,
        identifier: CredentialRequestIdentifier,
        credential_preview: &CredentialPreview,
        trust_anchors: &TrustAnchors,
        wscd: &W,
    ) -> Result<CredentialWithMetadata, WalletIssuanceError>
    where
        W: IssuanceWscd,
    {
        // Request as many copies as the Issuer Metadata will allow, capped at BATCH_SIZE_MAX.
        let copy_count = self.session_state.batch_size.into();

        // Fetch one nonce from the nonce endpoint, if defined in the issuer metadata. Use the DPoP nonce if it returns
        // one.
        let (proof_nonce, dpop_nonce) = match self.session_state.issuer_endpoints.nonce_endpoint.as_ref() {
            None => (None, self.session_state.dpop_nonce.clone()),
            Some(nonce_endpoint) => {
                let (NonceResponse { c_nonce }, dpop_nonce) = self
                    .message_client
                    .request_nonce(nonce_endpoint.clone().into_url())
                    .await?;

                // If the nonce endpoint response included a "DPoP-Nonce" header, return that value as the DPoP nonce.
                // Otherwise, use the value received in the Token Response, if any.
                let dpop_nonce = dpop_nonce.or_else(|| self.session_state.dpop_nonce.clone());

                (Some(c_nonce), dpop_nonce)
            }
        };

        // Have the WSCD generate as many private keys and proofs as the number of credential copies.
        let aud = self.session_state.credential_issuer.as_ref().to_string();
        let IssuanceResult { key_identifiers, pops } = wscd
            .perform_issuance(copy_count, aud, proof_nonce)
            .await
            .map_err(|e| WalletIssuanceError::PrivateKeyGeneration(e.into()))?;

        // Extract pairs of key identifiers and public keys and proofs from the WSCD response. Note that the WSCD may
        // have returned either fewer key identifiers or proofs than we requested, this iterator results in the
        // minimum of that.
        let key_ids_public_keys_and_proofs = key_identifiers
            .into_nonempty_iter()
            .zip(pops)
            .map(|(key_identifier, proof)| {
                // We assume here the WP gave us valid JWTs, and leave it up to the issuer to verify these.
                let header = proof
                    .dangerous_parse_header_unverified()
                    .map_err(WalletIssuanceError::JwtParse)?;

                let public_key = header.public_key().map_err(WalletIssuanceError::JwkConversion)?;

                Ok((key_identifier, public_key, proof))
            })
            .collect::<Result<VecNonEmpty<_>, WalletIssuanceError>>()?;

        let (key_ids_and_public_keys, proofs): (VecNonEmpty<_>, _) = key_ids_public_keys_and_proofs
            .into_nonempty_iter()
            .map(|(key_identifier, public_key, proof)| ((key_identifier, public_key), proof))
            .unzip();

        // Send the proofs of posession to the issuer in a Credential Request to actually fetch the credential copies.
        let url = self
            .session_state
            .issuer_endpoints
            .credential_endpoint
            .clone()
            .into_url();
        let dpop_header = self.session_state.dpop_header(url.clone(), &Method::POST, dpop_nonce)?;

        let credential_response = self
            .message_client
            .request_credential(
                url,
                &CredentialRequest::new(identifier, proofs),
                &dpop_header,
                &self.session_state.access_token,
            )
            .await?;

        // Extract the credentials from the request and verify each of the copies.
        let credentials = credential_response
            .into_immediate_credentials()
            .ok_or(WalletIssuanceError::DeferredIssuanceUnsupported)?;

        let type_metadata = self
            .session_state
            .type_metadata
            .get(&credential_preview.credential_payload.attestation_type)
            .expect("type constructor guarantees that metadata is present for all offered attestation types");

        let credential_copies = match credential_preview.format {
            Format::MsoMdoc => {
                let mdocs = credentials.into_issued_mdocs(
                    key_ids_and_public_keys,
                    &type_metadata.normalized_metadata,
                    credential_preview,
                    trust_anchors,
                )?;

                IssuedCredentialCopies::Mdoc(mdocs)
            }
            Format::SdJwt => {
                let sd_jwts = credentials.into_issued_sd_jwts(
                    key_ids_and_public_keys,
                    &type_metadata.normalized_metadata,
                    credential_preview,
                    trust_anchors,
                )?;

                IssuedCredentialCopies::SdJwt(sd_jwts)
            }
        };

        // Verify that all credentials contain the same metadata integrity value and validate this against the SD-JWT VC
        // Type Metadata document chain.
        let verified_metadata = credential_copies.verify_metadata_integrity(type_metadata.raw_metadata.clone())?;

        let credential_with_metadata = CredentialWithMetadata::new(
            credential_copies,
            credential_preview.credential_payload.attestation_type.clone(),
            credential_preview.credential_payload.expires,
            credential_preview.credential_payload.not_before,
            type_metadata.normalized_metadata.extended_vcts(),
            verified_metadata,
        );

        Ok(credential_with_metadata)
    }
}

impl<H: VcMessageClient> IssuanceSession for HttpIssuanceSession<H> {
    async fn accept_issuance<W>(
        &mut self,
        selection: &AcceptIssuanceSelection,
        trust_anchors: &TrustAnchors,
        wscd: &W,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd,
    {
        // Check if any passed indices are actually valid.
        if let AcceptIssuanceSelection::PreviewIndices(indices) = selection {
            let out_of_bounds = indices
                .iter()
                .copied()
                .filter(|index| *index >= self.session_state.offered_credentials.credential_count())
                .collect::<HashSet<_>>();

            if !out_of_bounds.is_empty() {
                return Err(WalletIssuanceError::AcceptSelectionOutOfBounds(out_of_bounds));
            }
        }

        // Fetch a set of credential copies for each credential in parallel.
        let credentials = try_join_all(
            self.session_state
                .offered_credentials
                .select_identifiers_and_previews(selection)
                .map(|(identifier, preview)| self.fetch_credential(identifier, preview, trust_anchors, wscd)),
        )
        .await?;

        Ok(credentials)
    }

    async fn reject_issuance(&self) -> Result<(), WalletIssuanceError> {
        let url = self
            .session_state
            .issuer_endpoints
            .credential_endpoint
            .clone()
            .into_url();

        let dpop_header =
            self.session_state
                .dpop_header(url.clone(), &Method::DELETE, self.session_state.dpop_nonce.clone())?;

        self.message_client
            .reject(url, &dpop_header, &self.session_state.access_token)
            .await?;

        Ok(())
    }

    fn previews_with_metadata(&self) -> impl Iterator<Item = (&CredentialPreview, &NormalizedTypeMetadata)> {
        self.session_state
            .offered_credentials
            .credential_previews()
            .map(|preview| {
                let metadata = self
                    .session_state
                    .type_metadata
                    .get(&preview.credential_payload.attestation_type)
                    .expect("type constructor guarantees that metadata is present for all offered attestation types");

                (preview, &metadata.normalized_metadata)
            })
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        &self.session_state.issuer_registration
    }
}

impl Credentials {
    /// Create a set of mdoc credentials out of the credential response. This also verifies the credentials.
    fn into_issued_mdocs(
        self,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        normalized_type_metadata: &NormalizedTypeMetadata,
        preview: &CredentialPreview,
        trust_anchors: &TrustAnchors,
    ) -> Result<VecNonEmpty<MdocCopy>, WalletIssuanceError> {
        let Self::MsoMdoc(mdoc_credentials) = self else {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: Format::MsoMdoc,
                actual: self.format(),
            });
        };

        let mdocs = mdoc_credentials
            .into_nonempty_iter()
            .zip(key_identifiers_and_public_keys)
            .map(|(mdoc_credential, (key_identifier, public_key))| {
                let MdocCredential {
                    credential: issuer_signed,
                } = mdoc_credential;

                // Calculate the minimum of all the lengths of the random bytes included in the attributes of
                // `IssuerSigned`. If this value is too low, we should not accept the attributes.
                let min_random_len = issuer_signed.name_spaces.as_ref().and_then(|namespaces| {
                    namespaces
                        .as_ref()
                        .values()
                        .flat_map(|attributes| attributes.as_ref().iter().map(|TaggedBytes(item)| item.random.len()))
                        .min()
                });

                if let Some(min) = min_random_len
                    && min < ATTR_RANDOM_LENGTH
                {
                    return Err(WalletIssuanceError::AttributeRandomLength(min, ATTR_RANDOM_LENGTH));
                }

                let credential_issuer_certificate = issuer_signed
                    .issuer_auth
                    .x5chain()
                    .map_err(WalletIssuanceError::IssuerCertificate)?
                    .into_first();

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new(issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(WalletIssuanceError::MdocVerification)?;

                let issued_credential_payload = CredentialPayload::from_mdoc(mdoc.clone(), normalized_type_metadata)
                    .map_err(WalletIssuanceError::MdocCredentialPayload)?;

                Self::validate_credential(
                    preview,
                    &public_key,
                    issued_credential_payload,
                    &credential_issuer_certificate,
                )?;

                Ok(MdocCopy { key_identifier, mdoc })
            })
            .collect::<Result<_, _>>()?;

        Ok(mdocs)
    }

    /// Create a set of SD-JWT credentials out of the credential response. This also verifies the credentials.
    fn into_issued_sd_jwts(
        self,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        normalized_type_metadata: &NormalizedTypeMetadata,
        preview: &CredentialPreview,
        trust_anchors: &TrustAnchors,
    ) -> Result<VecNonEmpty<SdJwtCopy>, WalletIssuanceError> {
        let Self::SdJwt(sd_jwt_credentials) = self else {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: Format::SdJwt,
                actual: self.format(),
            });
        };

        let sd_jwts = sd_jwt_credentials
            .into_nonempty_iter()
            .zip(key_identifiers_and_public_keys)
            .map(|(sd_jwt_credential, (key_identifier, public_key))| {
                let SdJwtCredential {
                    credential: unverified_sd_jwt,
                } = sd_jwt_credential;

                let sd_jwt = unverified_sd_jwt
                    .into_verified_against_trust_anchors(trust_anchors, &TimeGenerator)
                    .map_err(WalletIssuanceError::SdJwtVerification)?;

                let issued_credential_payload = CredentialPayload::from_sd_jwt(sd_jwt.clone())
                    .map_err(WalletIssuanceError::SdJwtCredentialPayloadError)?;

                // Store claim paths to later use in validation of selective disclosability of claims. This prevents
                // cloning `issued_credential_payload`.
                let issued_claims = issued_credential_payload
                    .previewable_payload
                    .attributes
                    .claim_paths(AttributesTraversalBehaviour::OnlyLeaves);

                Self::validate_credential(
                    preview,
                    &public_key,
                    issued_credential_payload,
                    sd_jwt.issuer_leaf_certificate(),
                )?;

                // Verify whether each claims selective disclosability matches the metadata. This validation is SD-JWT
                // specific, and therefore cannot be part of `validate_credential`.
                Self::verify_selective_disclosability(&sd_jwt, issued_claims, normalized_type_metadata.clone())?;

                Ok(SdJwtCopy { key_identifier, sd_jwt })
            })
            .collect::<Result<_, WalletIssuanceError>>()?;

        Ok(sd_jwts)
    }

    fn validate_credential(
        preview: &CredentialPreview,
        holder_pubkey: &PublicKey,
        credential_payload: CredentialPayload,
        credential_issuer_certificate: &BorrowingCertificate,
    ) -> Result<(), WalletIssuanceError> {
        if credential_payload.confirmation_key.try_to_public_key()? != *holder_pubkey {
            return Err(WalletIssuanceError::PublicKeyMismatch);
        }

        // The issuer certificate inside the mdoc has to equal the one that the issuer previously announced
        // in the credential preview.
        if credential_issuer_certificate != &preview.issuer_certificate {
            return Err(WalletIssuanceError::IssuerMismatch);
        }

        // Check that our mdoc contains exactly the attributes the issuer said it would have.
        // Note that this also means that the mdoc's attributes must match the received metadata,
        // as both the metadata and attributes are the same as when we checked this for the preview.
        if credential_payload.previewable_payload != preview.credential_payload {
            return Err(WalletIssuanceError::IssuedCredentialMismatch {
                actual: Box::new(credential_payload.previewable_payload),
                expected: Box::new(preview.credential_payload.clone()),
            });
        }

        Ok(())
    }

    fn verify_selective_disclosability(
        sd_jwt: &VerifiedSdJwt,
        issued_claims: Vec<VecNonEmpty<ClaimPath>>,
        metadata: NormalizedTypeMetadata,
    ) -> Result<(), WalletIssuanceError> {
        let sd_metadata = metadata
            .into_presentation_components()
            .2
            .into_iter()
            .map(|md| (md.path.into_inner(), md.sd))
            .collect();

        // Iterate over the issued_claims, validating each element in the path against the metadata.
        // This implementation will ignore any (optional) claims that do exist in the metadata but are not issued.
        // Validating whether all required claims are issued is done by `validate_credential`.
        // This will also prevent traversing and decoding the same disclosures several times for nested disclosures.
        for issued_claim in issued_claims {
            Self::verify_claim_selective_disclosability(sd_jwt, issued_claim.as_slice(), &sd_metadata)?;
        }

        Ok(())
    }

    fn verify_claim_selective_disclosability(
        sd_jwt: &VerifiedSdJwt,
        claim_to_verify: &[ClaimPath],
        sd_metadata: &HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata>,
    ) -> Result<(), WalletIssuanceError> {
        sd_jwt
            .verify_selective_disclosability(claim_to_verify, sd_metadata)
            .map_err(DecoderError::ClaimStructure)?;

        Ok(())
    }
}

impl IssuedCredentialCopies {
    /// Verify that each credential copy contains the same metadata integrity value, use this to validate a SD-JWT VC
    /// Type Metadata document chain and return the resulting `VerifiedTypeMetadataDocuments` type.
    fn verify_metadata_integrity(
        &self,
        metadata_documents: SortedTypeMetadataDocuments,
    ) -> Result<VerifiedTypeMetadataDocuments, WalletIssuanceError> {
        // Verify that each of the resulting credentials contain exactly the same metadata integrity digest.
        let unique_integrities: HashSet<_> = match self {
            IssuedCredentialCopies::Mdoc(mdocs) => mdocs
                .iter()
                .map(|mdoc_copy| {
                    mdoc_copy
                        .mdoc
                        .type_metadata_integrity()
                        .map_err(WalletIssuanceError::Metadata)
                })
                .try_collect()?,
            IssuedCredentialCopies::SdJwt(sd_jwts) => sd_jwts
                .iter()
                .map(|sd_jwt_copy| {
                    sd_jwt_copy
                        .sd_jwt
                        .claims()
                        .vct_integrity
                        .as_ref()
                        .ok_or(WalletIssuanceError::MetadataIntegrityMissing)
                })
                .try_collect()?,
        };

        let integrity = unique_integrities
            .into_iter()
            .exactly_one()
            .map_err(|_| WalletIssuanceError::MetadataIntegrityInconsistent)?;

        // Check that the integrity hash received in the credential matches that of encoded JSON of the first metadata
        // document.
        let verified_metadata = metadata_documents
            .into_verified(integrity.clone())
            .map_err(WalletIssuanceError::MetadataIntegrityVerification)?;

        Ok(verified_metadata)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::num::NonZeroU8;
    use std::time::Duration;
    use std::vec;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::qualification::AttestationQualification;
    use attestation_types::status_claim::StatusClaim;
    use chrono::Utc;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::PID_ISSUER_CERT_DN;
    use crypto::server_keys::generate::mock::PID_ISSUER_CERT_SAN_URI;
    use crypto::x509::CertificateError;
    use derive_more::Debug;
    use futures::FutureExt;
    use jwt::jwk::jwk_to_public_key;
    use jwt::nonce::Nonce;
    use mdoc::utils::serialization::TaggedBytes;
    use mockall::predicate::eq;
    use rstest::rstest;
    use sd_jwt::builder::SignedSdJwt;
    use sd_jwt::claims::ClaimName;
    use sd_jwt::error::ClaimError;
    use sd_jwt::test::conceal_and_sign;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use serde_bytes::ByteBuf;
    use serde_json::json;
    use ssri::Integrity;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::IntoNonEmptyIterator;
    use utils::vec_nonempty;
    use wscd::mock_remote::MockRemoteWscd;
    use wscd::mock_remote::MockWiaClient;

    use super::*;
    use crate::authorization_details::AuthorizationDetails;
    use crate::credential::CredentialRequestProofs;
    use crate::errors::ErrorResponse;
    use crate::errors::RemoteErrorCode;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::issuer_identifier::IssuerUrl;
    use crate::metadata::issuer_metadata::CredentialFormat;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::metadata::well_known::WellKnownMetadata;
    use crate::preview::CredentialPreviewResponse;
    use crate::token::CredentialPreview;
    use crate::token::CredentialPreviewError;
    use crate::token::TokenResponse;
    use crate::token::TokenType;
    use crate::wallet_issuance::TypeMetadataChainError;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::mock::RecordingWiaClient;

    impl<H> HttpIssuanceSession<H> {
        pub fn batch_size(&self) -> NonZeroU8 {
            self.session_state.batch_size
        }
    }

    fn invalid_grant_error() -> WalletIssuanceError {
        WalletIssuanceError::TokenRequest(Box::new(ErrorResponse {
            error: RemoteErrorCode::Known(TokenErrorCode::InvalidGrant),
            error_description: None,
            error_uri: None,
        }))
    }

    #[test]
    fn map_pre_authorized_token_error_translates_only_pre_authorized_invalid_grant() {
        use crate::token::AuthorizationCode;

        let pre_authorized = TokenRequest::new_pre_authorized(AuthorizationCode::from("the-code".to_string()));
        let authorization_code = TokenRequest::new_authorization_code(
            AuthorizationCode::from("the-code".to_string()),
            "https://example.com/redirect".parse().unwrap(),
            "code-verifier".to_string(),
        );

        // Pre-authorized flow + invalid_grant is unambiguously "code no longer valid".
        assert_matches!(
            map_pre_authorized_token_error(invalid_grant_error(), &pre_authorized),
            WalletIssuanceError::PreAuthorizedCodeExpired
        );

        // Authorization-code flow: invalid_grant is shared with PKCE / client_id failures, so it must
        // not be translated.
        assert_matches!(
            map_pre_authorized_token_error(invalid_grant_error(), &authorization_code),
            WalletIssuanceError::TokenRequest(_)
        );

        // Any other error code in the pre-authorized flow is left untouched.
        let other = WalletIssuanceError::TokenRequest(Box::new(ErrorResponse {
            error: RemoteErrorCode::Known(TokenErrorCode::InvalidRequest),
            error_description: None,
            error_uri: None,
        }));
        assert_matches!(
            map_pre_authorized_token_error(other, &pre_authorized),
            WalletIssuanceError::TokenRequest(_)
        );
    }

    #[rstest]
    #[case(ClientAttestationChallengeMechanism::None, None, false)]
    #[case(
        ClientAttestationChallengeMechanism::Header(Nonce::from("header-challenge".to_string())),
        Some(Nonce::from("header-challenge".to_string())),
        false
    )]
    #[case(
        ClientAttestationChallengeMechanism::ChallengeEndpoint("https://example.com/challenge".parse().unwrap()),
        Some(Nonce::from("endpoint-challenge".to_string())),
        true
    )]
    fn test_create_client_auth_challenge_mechanism(
        #[case] mechanism: ClientAttestationChallengeMechanism,
        #[case] expected_challenge: Option<Nonce>,
        #[case] expect_challenge_request: bool,
    ) {
        let issuer_identifier: IssuerIdentifier = "https://example.com".parse().unwrap();
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier,
            vec![(
                config_id,
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_metadata.issuer_identifier().clone());
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();

        let mut mock_msg_client = MockVcMessageClient::new();
        // Fail the token request (the step right after the challenge is resolved and the WIA is issued) with an
        // unambiguous, recognizable error, so `create()` short-circuits there. This keeps the fixture minimal: only
        // `request_token` (and `request_challenge`) need to be mocked, without also having to mock the type
        // metadata/preview fetching, trust anchors, etc. that follow.
        mock_msg_client
            .expect_request_token()
            .once()
            .return_once(move |_url, _token_request, _dpop_header, _wia_disclosure| Err(invalid_grant_error()));
        if expect_challenge_request {
            mock_msg_client
                .expect_request_challenge()
                .once()
                .return_once(move |_url| Ok("endpoint-challenge".to_string().into()));
        }

        let wia_client = RecordingWiaClient::default();
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = TrustAnchors::from(&ca);

        let error = HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            batch_size,
            oauth_metadata.token_endpoint,
            mechanism,
            TokenRequest::new_mock(),
            &wia_client,
            &oauth_metadata.issuer,
            &trust_anchors,
        )
        .now_or_never()
        .unwrap()
        .expect_err("should fail at the (mocked) token request, after the challenge has been resolved");

        // The failure should occur at the (mocked) token request, confirming that the challenge resolution itself
        // succeeded and did not short-circuit `create()` earlier.
        assert_matches!(error, WalletIssuanceError::PreAuthorizedCodeExpired);
        assert_eq!(*wia_client.received_challenge.borrow(), Some(expected_challenge));
    }

    #[derive(Debug, Clone)]
    enum TokenResponseFields {
        // Contains a list of credential config ids, each with a list of credential ids.
        AuthorizationDetails(Vec<(&'static str, Vec<&'static str>)>),
        // Contains a list of credential config ids.
        Scope(Vec<&'static str>),
        // Contains a combination of the two above variants.
        Both(Vec<(&'static str, Vec<&'static str>)>, Vec<&'static str>),
        Neither,
    }

    fn test_start_issuance(
        ca: &Ca,
        trust_anchors: &TrustAnchors,
        issuer_metadata: IssuerMetadata,
        preview_payloads: Vec<(String, CredentialConfigurationId, Format, PreviewableCredentialPayload)>,
        type_metadata: TypeMetadata,
        token_response_fields: &TokenResponseFields,
    ) -> Result<HttpIssuanceSession<MockVcMessageClient>, WalletIssuanceError> {
        let issuance_key = generate_pid_issuer_mock_with_registration(ca, &IssuerRegistration::new_mock()).unwrap();

        let authorization_details = match &token_response_fields {
            TokenResponseFields::AuthorizationDetails(identifiers) | TokenResponseFields::Both(identifiers, _) => {
                let (config_ids, credential_ids): (Vec<_>, Vec<_>) = identifiers
                    .iter()
                    .flat_map(|(config_id, credential_ids)| {
                        credential_ids.iter().map(|credential_id| {
                            (
                                CredentialConfigurationId::from(config_id.to_string()),
                                credential_id.to_string(),
                            )
                        })
                    })
                    .unzip();
                let credential_ids_and_identifiers =
                    VecNonEmpty::try_from(config_ids.iter().zip(credential_ids).collect_vec()).unwrap();

                Some(AuthorizationDetails::from_credential_ids_and_identifiers(
                    credential_ids_and_identifiers,
                ))
            }
            TokenResponseFields::Scope(_) | TokenResponseFields::Neither => None,
        };

        let scope = match &token_response_fields {
            TokenResponseFields::Scope(scope) | TokenResponseFields::Both(_, scope) => {
                Some(scope.iter().map(|value| value.to_string().parse().unwrap()).collect())
            }
            TokenResponseFields::AuthorizationDetails(_) | TokenResponseFields::Neither => None,
        };

        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client.expect_request_token().return_once(
            move |_url, _token_request, _dpop_header, _wia_disclosure| {
                let token_response = TokenResponse {
                    access_token: "access_token".to_string().into(),
                    token_type: TokenType::DPoP,
                    expires_in: None,
                    refresh_token: None,
                    scope,
                    authorization_details,
                };

                Ok((token_response, None))
            },
        );
        mock_msg_client
            .expect_request_challenge()
            .return_once(move |_url| Ok("challenge".to_string().into()));
        mock_msg_client.expect_request_type_metadata().returning(move |_url| {
            let (_, _, metadata_documents) = TypeMetadataDocuments::from_single_example(type_metadata.clone());

            Ok(metadata_documents)
        });

        mock_msg_client
            .expect_request_credential_preview()
            .return_once(move |_url, _access_token| {
                let previews = preview_payloads
                    .into_iter()
                    .map(
                        |(credential_id, config_id, format, preview_payload)| CredentialPreview {
                            credential_id,
                            config_id,
                            format,
                            credential_payload: preview_payload,
                            issuer_certificate: issuance_key.certificate().clone(),
                        },
                    )
                    .collect_vec()
                    .try_into()
                    .unwrap();

                Ok(CredentialPreviewResponse {
                    credential_previews: previews,
                })
            });

        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_metadata.issuer_identifier().clone());

        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            batch_size,
            oauth_metadata.token_endpoint,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(oauth_metadata.challenge_endpoint.unwrap()),
            TokenRequest::new_mock(),
            &MockWiaClient::new(),
            &oauth_metadata.issuer,
            trust_anchors,
        )
        .now_or_never()
        .unwrap()
    }

    #[rstest]
    #[case::authorization_details(
        TokenResponseFields::AuthorizationDetails(vec![("config_id", vec!["credential_id"])]),
        false
    )]
    #[case::authorization_details_extra_configs(
        TokenResponseFields::AuthorizationDetails(vec![("config_id", vec!["credential_id"])]),
        true
    )]
    #[case::scope(TokenResponseFields::Scope(vec!["config_id_scope"]), false)]
    #[case::scope_extra_configs(TokenResponseFields::Scope(vec!["config_id_scope"]), true)]
    #[case::authorization_details_and_scope(
        TokenResponseFields::Both(vec![("config_id", vec!["credential_id"])], vec!["config_id_scope"]),
        false
    )]
    #[case::authorization_details_and_invalid_scope(
        TokenResponseFields::Both(vec![("config_id", vec!["credential_id"])], vec!["invalid_scope"]),
        false
    )]
    #[case::authorization_details_and_scope_extra_configs(
        TokenResponseFields::Both(vec![("config_id", vec!["credential_id"])], vec!["config_id_scope"]),
        true
    )]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither, false)]
    // Note that the credential configurations cannot be limited if the Token Response contains neither
    // `authorization_details` nor `scope`.
    fn test_start_issuance_ok(
        #[case] token_response_fields: TokenResponseFields,
        #[case] has_extra_credential_configs: bool,
    ) {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let mut credential_configs = vec![(
            config_id.clone(),
            CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
        )];

        if has_extra_credential_configs {
            credential_configs.push((
                CredentialConfigurationId::from("other_config_id".to_string()),
                CredentialKind::new(Format::SdJwt, "other_vct".to_string()),
            ))
        }

        let session = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), credential_configs),
            vec![(
                "credential_id".to_string(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect("starting issuance session should succeed");

        let Ok((preview, metadata)) = session.previews_with_metadata().exactly_one() else {
            panic!("issuance session should contain exactly one preview")
        };

        assert_matches!(
                &preview.credential_payload.attributes.as_ref()["family_name"],
                Attribute::Single(AttributeValue::Text(v)) if v == "De Bruijn");

        assert_eq!(
            *metadata,
            TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example())
                .2
                .into_normalized(&preview.credential_payload.attestation_type)
                .unwrap()
                .0
        );
    }

    #[test]
    fn test_start_issuance_token_response_unknown_credential_config_ids() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string(),
                CredentialConfigurationId::from("config_id".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::AuthorizationDetails(vec![("unknown_config_id", vec!["credential_id"])]),
        )
        .expect_err("starting issuance session should fail");

        assert_matches!(
            error,
            WalletIssuanceError::TokenResponseUnknownCredentialConfigIds(config_ids)
                if config_ids == vec![CredentialConfigurationId::from("unknown_config_id".to_string())]
        );
    }

    #[test]
    fn test_start_issuance_token_response_empty_scope() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string(),
                CredentialConfigurationId::from("config_id".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Scope(vec![]),
        )
        .expect_err("starting issuance session should fail");

        assert_matches!(error, WalletIssuanceError::TokenResponseEmptyScope);
    }

    #[test]
    fn test_start_issuance_token_response_unknown_scope() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string(),
                CredentialConfigurationId::from("config_id".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Scope(vec!["unknown_config_id_scope"]),
        )
        .expect_err("starting issuance session should fail");

        assert_matches!(
            error,
            WalletIssuanceError::TokenResponseUnknownScope(scopes)
                if scopes == vec!["unknown_config_id_scope".parse().unwrap()]
        );
    }

    #[test]
    fn test_start_issuance_untrusted_credential_preview() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();

        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&other_ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    config_id.clone(),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Neither,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::CredentialPreviewVerification(CredentialPreviewError::Certificate(
                CertificateError::Verification(_)
            ))
        );
    }

    #[test]
    fn test_start_issuance_type_metadata_verification_error() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    config_id.clone(),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::example_empty(PID_ATTESTATION_TYPE, &MockTimeGenerator::default()),
            )],
            TypeMetadata::empty_example_with_attestation_type("other_attestation_type"),
            &TokenResponseFields::Neither,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::TypeMetadataVerification(_));
    }

    #[test]
    fn test_start_issuance_type_metadata_uri_missing() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        // Create issuer metadata with missing type_metadata_uri.
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let mut issuer_metadata = IssuerMetadata::new_mock(
            "https://example.com".parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        issuer_metadata
            .credential_configurations_supported
            .values_mut()
            .for_each(|config| config.type_metadata_uri = None);

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            issuer_metadata,
            vec![(
                "credential_id".to_string(),
                config_id.clone(),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Neither,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::TypeMetadataUriMissing(missing_config_ids) if missing_config_ids == vec![config_id]
        );
    }

    #[test]
    fn test_start_issuance_type_metadata_host_mismatch() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        // Create issuer metadata with incorrect type_metadata_uri.
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let issuer_metadata = IssuerMetadata::new_mock(
            "https://example.com".parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        let type_metadata_uri = IssuerUrl::try_new("https://metadata.example.com").unwrap();
        let mut config = issuer_metadata.credential_configurations_supported[&config_id].clone();
        config.type_metadata_uri = Some(type_metadata_uri.clone());
        let issuer_metadata = IssuerMetadata {
            credential_configurations_supported: [(config_id.clone(), config)].into(),
            ..issuer_metadata
        };

        let configured_issuer_identifier = issuer_metadata.credential_issuer.clone();
        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            issuer_metadata,
            vec![(
                "credential_id".to_string(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Neither,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::TypeMetadataHostMismatch(issuer_identifier, uris)
                if *issuer_identifier == configured_issuer_identifier && *uris.as_ref() == vec![type_metadata_uri]
        );
    }

    #[test]
    fn test_start_issuance_type_metadata_multiple_attestation_types() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        // Create issuer metadata with a type_metadata_uri that is used by two distinct credential configurations.
        let pid_config_id = CredentialConfigurationId::from("pid_config_id".to_string());
        let address_config_id = CredentialConfigurationId::from("address_config_id".to_string());

        let mut issuer_metadata = IssuerMetadata::new_mock(
            "https://example.com".parse().unwrap(),
            vec![(
                pid_config_id.clone(),
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        let mut address_credential_config = issuer_metadata
            .credential_configurations_supported
            .get(&pid_config_id)
            .unwrap()
            .clone();
        let expected_type_metadata_uri = address_credential_config.type_metadata_uri.clone().unwrap();
        let CredentialFormat::SdJwt { vct, .. } = &mut address_credential_config.format else {
            unreachable!()
        };
        *vct = ADDRESS_ATTESTATION_TYPE.to_string();
        issuer_metadata
            .credential_configurations_supported
            .insert(address_config_id.clone(), address_credential_config);

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            issuer_metadata,
            vec![
                (
                    "pid_credential_id".to_string(),
                    pid_config_id,
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "address_credential_id".to_string(),
                    address_config_id,
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_address_example(&MockTimeGenerator::default()),
                ),
            ],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Neither,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::TypeMetadataUriMultipleAttestationTypes(multi_attestation_type_uris)
                if multi_attestation_type_uris.len() == 1 &&
                    multi_attestation_type_uris.first().unwrap().0 == expected_type_metadata_uri &&
                    multi_attestation_type_uris
                        .first()
                        .unwrap()
                        .1
                        .iter()
                        .map(String::as_str)
                        .sorted()
                        .eq([ADDRESS_ATTESTATION_TYPE, PID_ATTESTATION_TYPE])
        );
    }

    #[rstest]
    #[case::authorization_details(
        TokenResponseFields::AuthorizationDetails(vec![
            ("config_id_1", vec!["credential_id_1"]),
            ("config_id_2", vec!["credential_id_2"])
        ])
    )]
    #[case::scope(TokenResponseFields::Scope(vec!["config_id_1_scope", "config_id_2_scope"]))]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither)]
    fn test_start_issuance_error_preview_missing_credential(#[case] token_response_fields: TokenResponseFields) {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![
                    (
                        CredentialConfigurationId::from("config_id_1".to_string()),
                        CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                    ),
                    (
                        CredentialConfigurationId::from("config_id_2".to_string()),
                        CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                    ),
                ],
            ),
            vec![(
                "credential_id_1".to_string(),
                CredentialConfigurationId::from("config_id_1".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect_err("starting issuance session should fail");

        let expected_credential_id = match token_response_fields {
            TokenResponseFields::AuthorizationDetails(_) | TokenResponseFields::Both(_, _) => {
                Some("credential_id_2".to_string())
            }
            TokenResponseFields::Scope(_) | TokenResponseFields::Neither => None,
        };
        let expected_missing = HashSet::from([("config_id_2".to_string().into(), expected_credential_id)]);
        assert_matches!(
            error,
            WalletIssuanceError::PreviewMissingCredentials(missing) if missing == expected_missing
        );
    }

    #[rstest]
    #[case::authorization_details(
        TokenResponseFields::AuthorizationDetails(vec![("config_id_1", vec!["credential_id_1_1"])])
    )]
    #[case::scope(TokenResponseFields::Scope(vec!["config_id_1_scope"]))]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither)]
    fn test_start_issuance_error_preview_excess_credentials(#[case] token_response_fields: TokenResponseFields) {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let error = test_start_issuance(
            &ca,
            &TrustAnchors::from(&ca),
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![
                (
                    "credential_id_1_1".to_string(),
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_2_1".to_string(),
                    CredentialConfigurationId::from("config_id_2".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_1_2".to_string(),
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
            ],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect_err("starting issuance session should fail");

        let expected_excess = vec![
            ("config_id_2".to_string().into(), "credential_id_2_1".to_string()),
            ("config_id_1".to_string().into(), "credential_id_1_2".to_string()),
        ];
        assert_matches!(error, WalletIssuanceError::PreviewExcessCredentials(excess) if excess == expected_excess);
    }

    #[test]
    fn test_start_issuance_error_different_issuer() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let issuer_registration = IssuerRegistration::new_mock();
        let issuance_key = generate_pid_issuer_mock_with_registration(&ca, &issuer_registration).unwrap();
        let different_issuance_key = {
            let mut different_dn = PID_ISSUER_CERT_DN.clone();
            different_dn.organization_name = Some("Different B.V.".to_string());
            ca.generate_key_pair(
                different_dn,
                issuer_registration.to_certificate_configuration().unwrap(),
                [PID_ISSUER_CERT_SAN_URI.clone()],
            )
            .unwrap()
        };

        let credential_id_mdoc = "credential_id_mdoc".to_string();
        let config_id_mdoc: CredentialConfigurationId = "config_id_mdoc".to_string().into();
        let credential_id_sd_jwt = "credential_id_sd_jwt".to_string();
        let config_id_sd_jwt: CredentialConfigurationId = "config_id_sd_jwt".to_string().into();
        let issuer_identifier: IssuerIdentifier = "https://issuer.example.com".parse().unwrap();
        let issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier.clone(),
            vec![
                (
                    config_id_mdoc.clone(),
                    CredentialKind::new(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string()),
                ),
                (
                    config_id_sd_jwt.clone(),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                ),
            ],
        );
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);

        let authorization_details = AuthorizationDetails::from_credential_ids_and_identifiers(vec_nonempty![
            (&config_id_mdoc, credential_id_mdoc.clone()),
            (&config_id_sd_jwt, credential_id_sd_jwt.clone())
        ]);
        let preview_payload =
            PreviewableCredentialPayload::example_empty(PID_ATTESTATION_TYPE, &MockTimeGenerator::default());

        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client.expect_request_token().return_once(
            move |_url, _token_request, _dpop_header, _wia_disclosure| {
                let token_response =
                    TokenResponse::new_vci("access_token".to_string().into(), Some(authorization_details));

                Ok((token_response, None))
            },
        );
        mock_msg_client
            .expect_request_challenge()
            .return_once(move |_url| Ok("challenge".to_string().into()));
        mock_msg_client
            .expect_request_type_metadata()
            .returning(|_url| Ok(TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example()).2));
        mock_msg_client
            .expect_request_credential_preview()
            .return_once(move |_url, _access_token| {
                let (_, _, _type_metadata) = TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example());

                let previews = vec_nonempty![
                    CredentialPreview {
                        credential_id: credential_id_mdoc,
                        config_id: config_id_mdoc.clone(),
                        format: Format::MsoMdoc,
                        credential_payload: preview_payload.clone(),
                        issuer_certificate: issuance_key.certificate().clone(),
                    },
                    CredentialPreview {
                        credential_id: credential_id_sd_jwt,
                        config_id: config_id_sd_jwt.clone(),
                        format: Format::SdJwt,
                        credential_payload: preview_payload,
                        issuer_certificate: different_issuance_key.certificate().clone(),
                    },
                ];

                Ok(CredentialPreviewResponse {
                    credential_previews: previews,
                })
            });

        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let error = HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            batch_size,
            oauth_metadata.token_endpoint,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(oauth_metadata.challenge_endpoint.unwrap()),
            TokenRequest::new_mock(),
            &MockWiaClient::new(),
            &oauth_metadata.issuer,
            &TrustAnchors::from(&ca),
        )
        .now_or_never()
        .unwrap()
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::DifferentIssuers);
    }

    /// Return a new session ready for `accept_issuance()`.
    fn new_session_state(
        credential_previews: Vec<CredentialPreview>,
        type_metadata: Vec<IssuanceTypeMetadata>,
        batch_size: NonZeroU8,
        has_nonce_endpoint: bool,
    ) -> IssuanceState {
        let issuer_identifier = "https://issuer.example.com".parse().unwrap();

        let mut issuer_endpoints = IssuerEndpoints::new_mock(&issuer_identifier);
        if !has_nonce_endpoint {
            issuer_endpoints.nonce_endpoint = None;
        }

        let previews_by_credential_id = credential_previews
            .into_iter()
            .map(|preview| (preview.credential_id.clone(), preview))
            .collect();

        let type_metadata = type_metadata
            .into_iter()
            .map(|metadata| (metadata.normalized_metadata.vct().to_string(), metadata))
            .collect();

        IssuanceState {
            access_token: "access_token".to_string().into(),
            credential_issuer: issuer_identifier,
            issuer_endpoints,
            batch_size,
            offered_credentials: OfferedCredentials::CredentialIds(previews_by_credential_id),
            type_metadata,
            issuer_registration: IssuerRegistration::new_mock(),
            dpop_signing_key: SigningKey::generate(),
            dpop_nonce: Some("dpop_nonce".parse().unwrap()),
        }
    }

    fn mock_openid_message_client_nonce(dpop_nonce: Option<&'static str>, call_count: usize) -> MockVcMessageClient {
        let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client
            .expect_request_nonce()
            .with(eq(Url::parse("https://issuer.example.com/issuance/nonce").unwrap()))
            .times(call_count)
            .returning(move |_| {
                Ok((
                    NonceResponse {
                        c_nonce: Nonce::from("c_nonce".to_string()),
                    },
                    dpop_nonce.map(|dpop_nonce| dpop_nonce.parse().unwrap()),
                ))
            });

        mock_msg_client
    }

    #[derive(Debug, Clone)]
    struct MockCredentialSigner {
        formats_by_credential_id: HashMap<String, Format>,
        pub trust_anchors: TrustAnchors,
        issuer_key: KeyPair,
        metadata_integrity: Integrity,
        pub first_metadata_integrity_random: bool,
        normalized_metadata: NormalizedTypeMetadata,
        preview_payload: PreviewableCredentialPayload,
    }

    impl MockCredentialSigner {
        pub fn new_with_preview_and_type_metadata(
            formats_by_credential_id: HashMap<String, Format>,
        ) -> (Self, Vec<CredentialPreview>, IssuanceTypeMetadata) {
            let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());
            let type_metadata = TypeMetadata::example_with_claim_name(&preview_payload.attestation_type, "family_name");

            Self::from_metadata_and_preview(formats_by_credential_id, type_metadata, preview_payload)
        }

        pub fn from_metadata_and_preview(
            formats_by_credential_id: HashMap<String, Format>,
            type_metadata: TypeMetadata,
            preview_payload: PreviewableCredentialPayload,
        ) -> (Self, Vec<CredentialPreview>, IssuanceTypeMetadata) {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let trust_anchors = TrustAnchors::try_from(vec![ca.to_borrowing_trust_anchor()]).unwrap();

            let issuer_registration = IssuerRegistration::new_mock();
            let issuer_key = generate_pid_issuer_mock_with_registration(&ca, &issuer_registration).unwrap();
            let issuer_certificate = issuer_key.certificate().clone();

            let (attestation_type, metadata_integrity, metadata_documents) =
                TypeMetadataDocuments::from_single_example(type_metadata);
            let (normalized_metadata, raw_metadata) = metadata_documents.into_normalized(&attestation_type).unwrap();

            let credential_count = formats_by_credential_id.len();
            let previews = formats_by_credential_id
                .clone()
                .into_iter()
                .zip(std::iter::repeat_n(
                    (preview_payload.clone(), issuer_certificate),
                    credential_count,
                ))
                .map(
                    |((credential_id, format), (credential_payload, issuer_certificate))| CredentialPreview {
                        credential_id,
                        config_id: "config_id".to_string().into(),
                        format,
                        credential_payload,
                        issuer_certificate,
                    },
                )
                .collect();

            let issuance_type_metadata = IssuanceTypeMetadata {
                normalized_metadata: normalized_metadata.clone(),
                raw_metadata,
            };

            let signer = Self {
                formats_by_credential_id,
                trust_anchors,
                issuer_key,
                metadata_integrity,
                first_metadata_integrity_random: false,
                normalized_metadata,
                preview_payload,
            };

            (signer, previews, issuance_type_metadata)
        }

        pub fn response_from_request(&self, request: &CredentialRequest) -> CredentialResponse {
            let credential_id = match &request.identifier {
                CredentialRequestIdentifier::CredentialIdentifier(credential_id) => credential_id,
                CredentialRequestIdentifier::CredentialConfigurationId(config_id) => {
                    if config_id.as_ref() != "config_id" {
                        panic!("requested credential configuration identifier is not correct");
                    }

                    self.formats_by_credential_id
                        .keys()
                        .exactly_one()
                        .expect("credential configuration identifier is only allowed for a single credential")
                }
            };

            let holder_pubkeys = match request.proofs.as_ref().unwrap() {
                CredentialRequestProofs::Jwt(jwts) => jwts
                    .nonempty_iter()
                    .map(|jwt| jwk_to_public_key(&jwt.dangerous_parse_header_unverified().unwrap().jwk).unwrap())
                    .collect::<VecNonEmpty<_>>(),
            };

            self.response_from_holder_pubkeys(credential_id, holder_pubkeys.nonempty_iter())
        }

        pub fn response_from_holder_pubkeys<'a>(
            &self,
            credential_id: &str,
            holder_pubkeys: impl IntoNonEmptyIterator<Item = &'a PublicKey>,
        ) -> CredentialResponse {
            let credential_payloads = holder_pubkeys
                .into_nonempty_iter()
                .enumerate()
                .map(|(index, holder_pubkey)| {
                    let metadata_integrity = if self.first_metadata_integrity_random && index == 0 {
                        Integrity::from(crypto::utils::random_bytes(32))
                    } else {
                        self.metadata_integrity.clone()
                    };

                    CredentialPayload::from_previewable_credential_payload_unvalidated(
                        self.preview_payload.clone(),
                        Utc::now(),
                        holder_pubkey,
                        metadata_integrity,
                        StatusClaim::new_mock(),
                    )
                    .unwrap()
                });

            let credentials = match self
                .formats_by_credential_id
                .get(credential_id)
                .expect("requested credential identifier is not correct")
            {
                Format::MsoMdoc => {
                    let mdoc_credentials = credential_payloads
                        .map(|credential_payload| {
                            let (issuer_signed, _) = credential_payload
                                .into_signed_mdoc(&self.issuer_key)
                                .now_or_never()
                                .unwrap()
                                .unwrap();

                            MdocCredential {
                                credential: issuer_signed,
                            }
                        })
                        .collect();

                    Credentials::MsoMdoc(mdoc_credentials)
                }
                Format::SdJwt => {
                    let sd_jwt_credentials = credential_payloads
                        .map(|credential_payload| {
                            let unverified_sd_jwt = credential_payload
                                .into_signed_sd_jwt(&self.normalized_metadata, &self.issuer_key)
                                .now_or_never()
                                .unwrap()
                                .unwrap()
                                .into();

                            SdJwtCredential {
                                credential: unverified_sd_jwt,
                            }
                        })
                        .collect();

                    Credentials::SdJwt(sd_jwt_credentials)
                }
            };

            CredentialResponse::new_immediate(credentials)
        }
    }

    /// Check consistency and validity of the input of the credential endpoint.
    fn check_credential_endpoint_input(
        url: &Url,
        dpop_signing_key: &SigningKey,
        dpop_nonce: &DpopNonce,
        dpop_header: Dpop,
        access_token: &AccessToken,
    ) {
        assert_eq!(access_token.as_ref(), "access_token");

        dpop_header
            .verify_expecting_key(
                PublicKey::from(*dpop_signing_key.verifying_key()),
                url,
                &Method::POST,
                Some(&"access_token".to_string().into()),
                Some(dpop_nonce),
            )
            .unwrap();
    }

    enum TestNonceEndpoint {
        Absent,
        Present,
        PresentWithDpopNonce,
    }

    enum AcceptIssuanceTestFormats {
        CredentialId(Vec<Format>),
        CredentialConfigurationId(Format),
    }

    #[rstest]
    #[case::credential_id_single_mdoc(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::MsoMdoc]),
        AcceptIssuanceSelection::All
    )]
    #[case::config_id_single_mdoc(
        AcceptIssuanceTestFormats::CredentialConfigurationId(Format::MsoMdoc),
        AcceptIssuanceSelection::All
    )]
    #[case::multi_mdoc(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::MsoMdoc, Format::MsoMdoc, Format::MsoMdoc]),
        AcceptIssuanceSelection::All
    )]
    #[case::credential_id_single_sd_jwt(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt]),
        AcceptIssuanceSelection::All
    )]
    #[case::config_id_single_sd_jwt(
        AcceptIssuanceTestFormats::CredentialConfigurationId(Format::SdJwt),
        AcceptIssuanceSelection::All
    )]
    #[case::multi_sd_jwt(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt, Format::SdJwt, Format::SdJwt]),
        AcceptIssuanceSelection::All
    )]
    #[case::mixed(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt, Format::MsoMdoc, Format::MsoMdoc, Format::SdJwt]),
        AcceptIssuanceSelection::All
    )]
    #[case::mixed_selection(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt, Format::MsoMdoc, Format::MsoMdoc, Format::SdJwt]),
        AcceptIssuanceSelection::PreviewIndices(HashSet::from([1, 3]))
    )]
    fn test_accept_issuance(
        #[case] formats: AcceptIssuanceTestFormats,
        #[case] selection: AcceptIssuanceSelection,
        #[values(NonZeroU8::MIN, 4.try_into().unwrap())] batch_size: NonZeroU8,
        #[values(
            TestNonceEndpoint::Absent,
            TestNonceEndpoint::Present,
            TestNonceEndpoint::PresentWithDpopNonce
        )]
        nonce_endpoint: TestNonceEndpoint,
    ) {
        let formats_by_credential_id = match formats {
            AcceptIssuanceTestFormats::CredentialId(formats) => formats
                .into_iter()
                .enumerate()
                .map(|(index, format)| (format!("credential_id_{index}"), format))
                .collect(),
            AcceptIssuanceTestFormats::CredentialConfigurationId(format) => {
                HashMap::from([("credential_id".to_string(), format)])
            }
        };

        let credential_count = match &selection {
            AcceptIssuanceSelection::All => formats_by_credential_id.len(),
            AcceptIssuanceSelection::PreviewIndices(indices) => indices.len(),
        };

        let (signer, previews, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata(formats_by_credential_id);
        let trust_anchors = signer.trust_anchors.clone();
        let wscd = MockRemoteWscd::default();

        let (mut mock_msg_client, has_nonce_endpoint, expected_dpop_nonce) = match nonce_endpoint {
            TestNonceEndpoint::Absent => (MockVcMessageClient::new(), false, "dpop_nonce".parse().unwrap()),
            TestNonceEndpoint::Present => (
                mock_openid_message_client_nonce(None, credential_count),
                true,
                "dpop_nonce".parse().unwrap(),
            ),
            TestNonceEndpoint::PresentWithDpopNonce => (
                mock_openid_message_client_nonce(Some("new_dpop_nonce"), credential_count),
                true,
                "new_dpop_nonce".parse().unwrap(),
            ),
        };

        let session_state = new_session_state(previews, vec![type_metadata], batch_size, has_nonce_endpoint);

        let dpop_signing_key = session_state.dpop_signing_key.clone();
        mock_msg_client
            .expect_request_credential()
            .times(credential_count)
            .returning(move |url, credential_request, dpop_header, access_token| {
                check_credential_endpoint_input(
                    &url,
                    &dpop_signing_key,
                    &expected_dpop_nonce,
                    dpop_header.clone(),
                    access_token,
                );

                Ok(signer.response_from_request(credential_request))
            });

        let credential_copies = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&selection, &trust_anchors, &wscd)
        .now_or_never()
        .unwrap()
        .expect("accepting issuance should succeed");

        assert_eq!(credential_copies.len(), credential_count);
    }

    #[test]
    fn test_accept_issuance_error_accept_selection_out_of_bounds() {
        let (signer, previews, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata(HashMap::from([
                ("credential_id_1".to_string(), Format::SdJwt),
                ("credential_id_2".to_string(), Format::SdJwt),
                ("credential_id_3".to_string(), Format::SdJwt),
            ]));

        let error = HttpIssuanceSession {
            message_client: MockVcMessageClient::new(),
            session_state: new_session_state(previews, vec![type_metadata], NonZeroU8::MIN, true),
        }
        .accept_issuance(
            &AcceptIssuanceSelection::PreviewIndices(HashSet::from([2, 3, 42])),
            &signer.trust_anchors,
            &MockRemoteWscd::default(),
        )
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::AcceptSelectionOutOfBounds(indices) if indices == HashSet::from([3, 42])
        );
    }

    #[test]
    fn test_accept_issuance_error_metadata_integrity_inconsistent() {
        let (mut signer, previews, type_metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id_1".to_string(), Format::SdJwt)]),
        );
        let trust_anchors = signer.trust_anchors.clone();

        // Prepare one of the returned SD-JWT copies to have an incorrect resource integrity in its payload.
        signer.first_metadata_integrity_random = true;

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().times(1).return_once(
            move |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.response_from_request(credential_request);

                Ok(response)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, vec![type_metadata], 4.try_into().unwrap(), true),
        }
        .accept_issuance(
            &AcceptIssuanceSelection::All,
            &trust_anchors,
            &MockRemoteWscd::default(),
        )
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::MetadataIntegrityInconsistent);
    }

    #[test]
    fn test_accept_issuance_error_metadata_integrity_verification() {
        let (mut signer, previews, type_metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string(), Format::SdJwt)]),
        );
        let trust_anchors = signer.trust_anchors.clone();

        // Include a random resource integrity in the payload of the returned SD-JWT.
        signer.first_metadata_integrity_random = true;

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().times(1).return_once(
            move |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.response_from_request(credential_request);

                Ok(response)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, vec![type_metadata], NonZeroU8::MIN, true),
        }
        .accept_issuance(
            &AcceptIssuanceSelection::All,
            &trust_anchors,
            &MockRemoteWscd::default(),
        )
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::MetadataIntegrityVerification(TypeMetadataChainError::ResourceIntegrity(_))
        );
    }

    #[rstest]
    fn test_accept_issuance_error_deferred_issuance_unsupported() {
        let (signer, previews, type_metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string(), Format::SdJwt)]),
        );

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().times(1).return_once(
            |_url, _credential_request, _dpop_header, _access_token| {
                let credential_response = CredentialResponse::Deferred {
                    transaction_id: "12345".to_string(),
                    interval: Duration::from_hours(24),
                };

                Ok(credential_response)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, vec![type_metadata], NonZeroU8::MIN, true),
        }
        .accept_issuance(
            &AcceptIssuanceSelection::All,
            &signer.trust_anchors,
            &MockRemoteWscd::default(),
        )
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::DeferredIssuanceUnsupported);
    }

    fn mock_credential_response_credential(
        format: Format,
    ) -> (
        Credentials,
        CredentialPreview,
        IssuanceTypeMetadata,
        PublicKey,
        TrustAnchors,
    ) {
        let (signer, previews, type_metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string(), format)]),
        );
        let holder_pubkey = PublicKey::from(*SigningKey::generate().verifying_key());
        let credential_response = signer
            .response_from_holder_pubkeys("credential_id", vec_nonempty![&holder_pubkey])
            .into_immediate_credentials()
            .unwrap();

        (
            credential_response,
            previews.into_iter().next().unwrap(),
            type_metadata,
            holder_pubkey,
            signer.trust_anchors,
        )
    }

    fn credentials_test_into_issued_credential(
        credentials: Credentials,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        normalized_type_metadata: &NormalizedTypeMetadata,
        preview: &CredentialPreview,
        trust_anchors: &TrustAnchors,
    ) -> Result<(), WalletIssuanceError> {
        match &credentials {
            Credentials::MsoMdoc(_) => credentials
                .into_issued_mdocs(
                    key_identifiers_and_public_keys,
                    normalized_type_metadata,
                    preview,
                    trust_anchors,
                )
                .map(|_| ()),
            Credentials::SdJwt(_) => credentials
                .into_issued_sd_jwts(
                    key_identifiers_and_public_keys,
                    normalized_type_metadata,
                    preview,
                    trust_anchors,
                )
                .map(|_| ()),
        }
    }

    #[rstest]
    fn test_credential_response_into_credential(#[values(Format::MsoMdoc, Format::SdJwt)] format: Format) {
        let (credentials, preview_data, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview_data,
            &trust_anchor,
        )
        .expect("should be able to convert CredentialResponse into credential");
    }

    #[test]
    fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credentials, preview_data, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(Format::MsoMdoc);

        // Converting a `CredentialResponse` into an `Mdoc` from a response
        // that contains insufficient random data should fail.
        let credentials = match credentials {
            Credentials::MsoMdoc(mdoc_credentials) => {
                let mdoc_credentials = mdoc_credentials
                    .into_nonempty_iter()
                    .map(
                        |MdocCredential {
                             credential: mut issuer_signed,
                         }| {
                            let name_spaces = issuer_signed.name_spaces.as_mut().unwrap();

                            name_spaces.modify_first_attributes(|attributes| {
                                let TaggedBytes(first_item) = attributes.first_mut().unwrap();

                                first_item.random = ByteBuf::from(b"12345");
                            });

                            MdocCredential::new(issuer_signed)
                        },
                    )
                    .collect();

                Credentials::MsoMdoc(mdoc_credentials)
            }
            Credentials::SdJwt(_) => panic!("unsupported credential request format"),
        };

        let error = credentials
            .into_issued_mdocs(
                vec_nonempty![("key_id".to_string(), holder_public_key)],
                &type_metadata.normalized_metadata,
                &preview_data,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH));
    }

    #[test]
    fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credentials, preview, type_metadata, holder_public_key, _) =
            mock_credential_response_credential(Format::MsoMdoc);

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credentials
            .into_issued_mdocs(
                vec_nonempty![("key_id".to_string(), holder_public_key)],
                &type_metadata.normalized_metadata,
                &preview,
                &TrustAnchors::empty(),
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::MdocVerification(_));
    }

    #[test]
    fn test_credential_response_into_sd_jwt_sd_jwt_verification_error() {
        let (credentials, preview, type_metadata, holder_public_key, _) =
            mock_credential_response_credential(Format::SdJwt);

        // Converting a `CredentialResponse` into an SD-JWT credential that
        // is validated against incorrect trust anchors should fail.
        let error = credentials
            .into_issued_sd_jwts(
                vec_nonempty![("key_id".to_string(), holder_public_key)],
                &type_metadata.normalized_metadata,
                &preview,
                &TrustAnchors::empty(),
            )
            .expect_err("should not be able to convert CredentialResponse into SD-JWT");

        assert_matches!(error, WalletIssuanceError::SdJwtVerification(_));
    }

    #[rstest]
    fn test_credential_response_into_mdoc_public_key_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, preview_data, type_metadata, _, trust_anchor) = mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = PublicKey::from(*SigningKey::generate().verifying_key());
        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), other_public_key)],
            &type_metadata.normalized_metadata,
            &preview_data,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::PublicKeyMismatch);
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_issuance_key =
            generate_pid_issuer_mock_with_registration(&other_ca, &IssuerRegistration::new_mock()).unwrap();
        let preview_data = CredentialPreview {
            issuer_certificate: other_issuance_key.certificate().clone(),
            ..preview
        };

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview_data,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuerMismatch);
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_attributes_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail.
        let attributes = PreviewableCredentialPayload::example_with_attributes(
            PID_ATTESTATION_TYPE,
            Attributes::example([
                (["new"], AttributeValue::Bool(true)),
                (["family_name"], AttributeValue::Text(String::from("De Bruijn"))),
            ]),
            &MockTimeGenerator::default(),
        )
        .attributes;
        preview.credential_payload.attributes = attributes;

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_issuer_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` with a different `issuer_uri` in the preview than
        // contained within the response should fail.
        preview.credential_payload.issuer = "https://other-issuer.example.com".parse().unwrap();

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_doctype_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        preview.credential_payload.attestation_type = String::from("other.attestation_type");

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.

        preview.credential_payload.not_before = Some((Utc::now() + chrono::Duration::days(1)).into());

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_attestation_qualification_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format);

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        preview.credential_payload.attestation_qualification = AttestationQualification::PubEAA;

        let error = credentials_test_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &type_metadata.normalized_metadata,
            &preview,
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    #[case(vec_nonempty![ClaimPath::SelectByKey("non_existing".to_string())], vec![], ExpectedResult::ObjectFieldNotFound("non_existing".parse().unwrap())
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())]], ExpectedResult::Ok
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_always".to_string())], vec![], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Always, false)
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())]], ExpectedResult::Ok
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_allow".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())]], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Never, true)
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_value_never".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())]], ExpectedResult::Ok
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_always".to_string())], vec![], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Always, false)
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())]], ExpectedResult::Ok
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_allow".to_string())], vec![], ExpectedResult::Ok)]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())], vec![vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())]], ExpectedResult::SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata::Never, true)
    )]
    #[case(vec_nonempty![ClaimPath::SelectByKey("root_array_never".to_string())], vec![], ExpectedResult::Ok)]
    fn test_verify_claim_selective_disclosability(
        #[case] claim_to_verify: VecNonEmpty<ClaimPath>,
        #[case] claims_to_conceal: Vec<VecNonEmpty<ClaimPath>>,
        #[case] expected: ExpectedResult,
    ) {
        let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = issuer_ca.generate_issuer_mock().unwrap();

        let claims_metadata: HashMap<Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata> = HashMap::from_iter([
            (
                vec![ClaimPath::SelectByKey("root_value_always".to_string())],
                ClaimSelectiveDisclosureMetadata::Always,
            ),
            (
                vec![ClaimPath::SelectByKey("root_value_allow".to_string())],
                ClaimSelectiveDisclosureMetadata::Allowed,
            ),
            (
                vec![ClaimPath::SelectByKey("root_value_never".to_string())],
                ClaimSelectiveDisclosureMetadata::Never,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_always".to_string())],
                ClaimSelectiveDisclosureMetadata::Always,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_allow".to_string())],
                ClaimSelectiveDisclosureMetadata::Allowed,
            ),
            (
                vec![ClaimPath::SelectByKey("root_array_never".to_string())],
                ClaimSelectiveDisclosureMetadata::Never,
            ),
        ]);

        let signed_sd_jwt: SignedSdJwt = conceal_and_sign(
            &issuer_keypair,
            serde_json::from_value(json!({
                "vct": "com:example:1",
                "iss": "https://issuer.example.com/",
                "iat": 1683000000,
                "cnf": {
                    "jwk": {
                        "kty": "EC",
                        "crv": "P-256",
                        "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
                        "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
                    }
                },
                "root_value_always": 1,
                "root_value_allow": 2,
                "root_value_never": 3,
                "root_array_always": [
                    4
                ],
                "root_array_allow": [
                    5
                ],
                "root_array_never": [
                    6
                ],
            }))
            .unwrap(),
            claims_to_conceal,
        );
        let sd_jwt: VerifiedSdJwt = signed_sd_jwt.into_verified();

        let result =
            Credentials::verify_claim_selective_disclosability(&sd_jwt, claim_to_verify.as_slice(), &claims_metadata);

        match expected {
            ExpectedResult::Ok => result.unwrap(),
            ExpectedResult::ObjectFieldNotFound(expected_claim_name) => {
                let error = result.unwrap_err();
                assert_matches!(error, WalletIssuanceError::SdJwtVerification(DecoderError::ClaimStructure(
                    ClaimError::ObjectFieldNotFound(claim_name, _)
                )) if claim_name == expected_claim_name);
            }
            ExpectedResult::SelectivelyDisclosability(expected_sd, expected_disclosability) => {
                let error = result.unwrap_err();
                assert_matches!(error, WalletIssuanceError::SdJwtVerification(DecoderError::ClaimStructure(
                    ClaimError::SelectiveDisclosabilityMismatch(claim, sd, is_selective_disclosable)))
                                if claim == claim_to_verify.into_inner()
                                && expected_sd == sd
                                && expected_disclosability == is_selective_disclosable);
            }
        }
    }

    enum ExpectedResult {
        Ok,
        ObjectFieldNotFound(ClaimName),
        SelectivelyDisclosability(ClaimSelectiveDisclosureMetadata, bool),
    }
}
