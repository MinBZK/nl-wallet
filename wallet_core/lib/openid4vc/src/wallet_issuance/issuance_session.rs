use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
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
use reqwest::header::ToStrError;
use sd_jwt::error::DecoderError;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;
use utils::generator::TimeGenerator;
use utils::single_unique::SingleUnique;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_at_least::VecNonEmptyUnique;
use wscd::wscd::IssuanceWscd;
use wscd::wscd::WiaClient;

use super::IssuanceSession;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use super::credential::IssuedCredentialCopies;
use super::credential::SdJwtCopy;
use crate::authorization_details::IssuerAuthorizationDetails;
use crate::client_auth::ClientAttestationChallengeMechanism;
use crate::client_auth::fetch_client_auth_challenge;
use crate::credential::CredentialResponse;
use crate::credential::Credentials;
use crate::credential::MdocCredential;
use crate::credential::SdJwtCredential;
use crate::credential::draft;
use crate::dpop::DPOP_HEADER_NAME;
use crate::dpop::DPOP_NONCE_HEADER_NAME;
use crate::dpop::Dpop;
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
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError>;

    async fn request_challenge(&self, url: Url) -> Result<Nonce, WalletIssuanceError>;

    async fn request_credential_preview(
        &self,
        url: &Url,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError>;

    async fn request_type_metadata(&self, url: Url) -> Result<TypeMetadataDocuments, WalletIssuanceError>;

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<String>), WalletIssuanceError>;

    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &draft::CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, WalletIssuanceError>;

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &draft::CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<draft::CredentialResponses, WalletIssuanceError>;

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), WalletIssuanceError>;
}

#[derive(Debug)]
pub struct HttpVcMessageClient {
    http_client: HttpClient,
}

impl HttpVcMessageClient {
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    fn dpop_nonce(response: &Response) -> Result<Option<String>, ToStrError> {
        let dpop_nonce = response
            .headers()
            .get(DPOP_NONCE_HEADER_NAME)
            .map(|val| val.to_str())
            .transpose()?
            .map(str::to_string);

        Ok(dpop_nonce)
    }
}

impl VcMessageClient for HttpVcMessageClient {
    async fn request_token(
        &self,
        url: &Url,
        token_request: &TokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header.to_string())
                    .header(WIA_HEADER_NAME, wia.wia().serialization())
                    .header(WIA_POP_HEADER_NAME, wia.wia_pop().serialization())
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
        url: &Url,
        access_token: &AccessToken,
    ) -> Result<CredentialPreviewResponse, WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| builder.bearer_auth(access_token.as_ref()))
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

    async fn request_nonce(&self, url: Url) -> Result<(NonceResponse, Option<String>), WalletIssuanceError> {
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
        url: &Url,
        credential_request: &draft::CredentialRequest,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<CredentialResponse, WalletIssuanceError> {
        self.request(url, credential_request, dpop_header, access_token_header)
            .await
    }

    async fn request_credentials(
        &self,
        url: &Url,
        credential_requests: &draft::CredentialRequests,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<draft::CredentialResponses, WalletIssuanceError> {
        self.request(url, credential_requests, dpop_header, access_token_header)
            .await
    }

    async fn reject(&self, url: &Url, dpop_header: &str, access_token_header: &str) -> Result<(), WalletIssuanceError> {
        self.http_client
            .delete(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header)
                    .header(AUTHORIZATION, access_token_header)
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

impl HttpVcMessageClient {
    async fn request<T: Serialize, S: DeserializeOwned>(
        &self,
        url: &Url,
        request: &T,
        dpop_header: &str,
        access_token_header: &str,
    ) -> Result<S, WalletIssuanceError> {
        self.http_client
            .post(url.as_ref(), |builder| {
                builder
                    .header(DPOP_HEADER_NAME, dpop_header)
                    .header(AUTHORIZATION, access_token_header)
                    .json(request)
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
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    batch_size: NonZeroU8,
    credential_previews: VecNonEmpty<CredentialPreview>,
    credential_request_types: VecNonEmpty<draft::CredentialRequestType>,
    type_metadata: HashMap<String, IssuanceTypeMetadata>,
    issuer_registration: IssuerRegistration,
    #[debug(skip)]
    dpop_signing_key: SigningKey,
    dpop_nonce: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuanceTypeMetadata {
    pub normalized_metadata: NormalizedTypeMetadata,
    pub raw_metadata: SortedTypeMetadataDocuments,
}

#[derive(Debug)]
struct OfferedCredentialConfig {
    config_id: CredentialConfigurationId,
    config: CredentialConfiguration,
    // TODO (PVW-5554): Include Credential Identifiers when requesting credentials at the Credential Endpoint.
    #[expect(unused)]
    credential_identifiers: Option<VecNonEmptyUnique<String>>,
}

impl OfferedCredentialConfig {
    pub fn new_without_identifiers(config_id: CredentialConfigurationId, config: CredentialConfiguration) -> Self {
        Self {
            config,
            config_id,
            credential_identifiers: None,
        }
    }

    pub fn new_with_identifiers(
        config_id: CredentialConfigurationId,
        config: CredentialConfiguration,
        credential_identifiers: VecNonEmptyUnique<String>,
    ) -> Self {
        Self {
            config,
            config_id,
            credential_identifiers: Some(credential_identifiers),
        }
    }
}

fn credential_request_types_from_preview(
    credential_previews: &VecNonEmpty<CredentialPreview>,
    batch_size: NonZeroU8,
) -> VecNonEmpty<draft::CredentialRequestType> {
    // The OpenID4VCI `/batch_credential` endpoints supports issuance of multiple attestations, but the protocol
    // has no support (yet) for issuance of multiple copies of multiple attestations.
    // We implement this below by simply flattening the relevant nested iterators when communicating with the
    // issuer.
    //
    // The `/batch_credential` endpoint also does not support reading the `CredentialRequest::credential_type`
    // field, it will simply provide the wallet with copies of all of the credential formats it proposes.
    // For this reason, it is simply an error if the wallet does not support all of the formats proposed by
    // the issuer.
    //
    // TODO (PVW-4366): Have the batch issuance endpoint consider the `credential_type` field
    //                  of the `CredentialRequest`s and only issue those formats.

    credential_previews
        .nonempty_iter()
        .flat_map(|preview| {
            let request_type = draft::CredentialRequestType::from_format(
                preview.format,
                preview.credential_payload.attestation_type.clone(),
            );

            // Construct a `Vec<CredentialRequestType>`, with one entry per copy for this credential.
            utils::vec_at_least::repeat_n(request_type, batch_size.into())
        })
        .collect()
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
        token_endpoint: &Url,
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

        // TODO (PVW-5554): Store the offered credential configurations in the `IssuanceState` in order to request these
        //                  from the new credential endpoint.

        let credential_config_iter = offered_credential_configs
            .iter()
            .map(|OfferedCredentialConfig { config_id, config, .. }| (config_id, config));

        // Request preview and fetch type metadata
        let (type_metadata, credential_previews) = try_join!(
            Self::fetch_type_metadata(credential_config_iter, &credential_issuer, &message_client),
            Self::request_previews(
                credential_preview_endpoint.as_url(),
                &token_response.access_token,
                &message_client
            )
        )?;

        let issuer_registration = credential_previews
            .iter()
            .map(|preview| preview.issuer_registration())
            .collect::<Result<Vec<_>, _>>()
            .map_err(WalletIssuanceError::PreviewIssuerRegistration)?
            .iter()
            .single_unique()
            .map_err(WalletIssuanceError::DifferentIssuers)?
            .expect("there are always credential_previews in the preview response")
            .clone();

        // Verify the issuer certificate against the trust anchors.
        for preview in &credential_previews {
            preview
                .verify(trust_anchors)
                .map_err(WalletIssuanceError::CredentialPreviewVerification)?;
        }

        let credential_request_types = credential_request_types_from_preview(&credential_previews, batch_size);

        let session_state = IssuanceState {
            access_token: token_response.access_token,
            credential_issuer,
            issuer_endpoints,
            batch_size,
            credential_previews,
            credential_request_types,
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
    ) -> Result<Vec<OfferedCredentialConfig>, WalletIssuanceError> {
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
            (None, None) => {
                let offered_configs = credential_configurations
                    .into_iter()
                    .map(|(config_id, config)| OfferedCredentialConfig::new_without_identifiers(config_id, config))
                    .collect();

                Ok(offered_configs)
            }
        }
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the
    /// `authorization_details` field.
    fn filter_credential_configs_authorization_details(
        mut credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        authorization_details: IssuerAuthorizationDetails,
    ) -> Result<Vec<OfferedCredentialConfig>, WalletIssuanceError> {
        let (offered_configs, unknown_config_ids): (_, Vec<_>) = authorization_details
            .into_credential_ids_and_identifiers()
            .into_iter()
            .partition_map(
                |(config_id, identifiers)| match credential_configurations.remove(&config_id) {
                    Some(config) => Either::Left(OfferedCredentialConfig::new_with_identifiers(
                        config_id,
                        config,
                        identifiers,
                    )),
                    None => Either::Right(config_id),
                },
            );

        if !unknown_config_ids.is_empty() {
            return Err(WalletIssuanceError::TokenResponseUnknownCredentialConfigIds(
                unknown_config_ids,
            ));
        }

        Ok(offered_configs)
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the `scope` field.
    fn filter_credential_configs_scope(
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        scope: &HashSet<Scope>,
    ) -> Result<Vec<OfferedCredentialConfig>, WalletIssuanceError> {
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
            .filter_map(|(config_id, config)| {
                config
                    .scope
                    .as_ref()
                    .is_some_and(|config_scope| scope.contains(config_scope))
                    .then_some(OfferedCredentialConfig::new_without_identifiers(config_id, config))
            })
            .collect();

        Ok(offered_configs)
    }

    async fn request_previews(
        preview_endpoint: &Url,
        access_token: &AccessToken,
        message_client: &H,
    ) -> Result<VecNonEmpty<CredentialPreview>, WalletIssuanceError> {
        let CredentialPreviewResponse { credential_previews } = message_client
            .request_credential_preview(preview_endpoint, access_token)
            .await?;

        // TODO (PVW-5558): At this point, we should check that the contents of the credential preview match exactly
        //                  what the issuer offered in the Token Response by means of either the `authorization_details`
        //                  or `scope` field. We defer implementation of this to when the new preview data structure
        //                  will be implemented.

        Ok(credential_previews)
    }

    async fn fetch_type_metadata(
        credential_configurations: impl IntoIterator<Item = (&CredentialConfigurationId, &CredentialConfiguration)>,
        credential_issuer: &IssuerIdentifier,
        message_client: &H,
    ) -> Result<HashMap<String, IssuanceTypeMetadata>, WalletIssuanceError> {
        // Get the metadata URI and attestation_type for each credential configuration, while collecting any Credential
        // Configuration IDs for which no type metadata URI is given.
        let (configs_data, missing_uri_config_ids): (Vec<_>, Vec<_>) = credential_configurations
            .into_iter()
            .partition_map(|(config_id, config)| match config.type_metadata_uri.as_ref() {
                Some(uri) => {
                    let attestation_type = config
                        .format
                        .attestation_type()
                        // TODO (PVW-6161): Handle unsupported formats earlier and more consistently.
                        .expect("unsupported format");

                    Either::Left((uri, attestation_type))
                }
                None => Either::Right(config_id.clone()),
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

        // Make sure there is only one attestation type per URI, while retaining the config IDs.
        let (attestation_types_and_uris, multi_attestation_type_uris): (Vec<_>, Vec<_>) = attestation_types_per_uri
            .into_iter()
            .partition_map(
                |(uri, attestation_types)| match attestation_types.into_iter().exactly_one() {
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
}

impl<H: VcMessageClient> IssuanceSession for HttpIssuanceSession<H> {
    async fn accept_issuance<W>(
        &mut self,
        trust_anchors: &TrustAnchors,
        wscd: &W,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd,
    {
        let issuer_endpoints = &self.session_state.issuer_endpoints;
        let key_count = self.session_state.credential_request_types.len();

        // Determine the correct credential endpoint URL, to be used below.
        let credential_endpoint_url = if key_count.get() == 1 {
            &issuer_endpoints.credential_endpoint
        } else {
            issuer_endpoints
                .batch_credential_endpoint
                .as_ref()
                .ok_or(WalletIssuanceError::NoBatchCredentialEndpoint)?
        }
        .as_url();

        // Fetch one nonce from the nonce endpoint, if defined in the issuer metadata.
        let c_nonce = match issuer_endpoints.nonce_endpoint.as_ref() {
            None => None,
            Some(nonce_endpoint) => {
                let (NonceResponse { c_nonce }, dpop_nonce) = self
                    .message_client
                    .request_nonce(nonce_endpoint.clone().into_url())
                    .await?;

                // If the nonce endpoint response included a DPoP-Nonce header, update the value in the state.
                if let Some(dpop_nonce) = dpop_nonce {
                    self.session_state.dpop_nonce = Some(dpop_nonce);
                }

                Some(c_nonce)
            }
        };

        let aud = self.session_state.credential_issuer.as_ref().to_string();

        let issuance_data = wscd
            .perform_issuance(key_count, aud.clone(), c_nonce.clone())
            .await
            .map_err(|e| WalletIssuanceError::PrivateKeyGeneration(e.into()))?;

        let proofs = issuance_data
            .pops
            .into_iter()
            .map(|jwt| draft::CredentialRequestProof::Jwt { jwt });

        // Call the amount of proofs we received N, which equals `key_count`.
        // Combining these with the key identifiers and attestation types, compute N public keys and
        // N credential requests.
        let (pubkeys, mut credential_requests): (Vec<_>, Vec<_>) = try_join_all(
            proofs
                .zip(issuance_data.key_identifiers.into_inner())
                .zip(self.session_state.credential_request_types.clone())
                .map(|((proof, id), credential_request_type)| async move {
                    let draft::CredentialRequestProof::Jwt { jwt } = &proof;

                    // We assume here the WP gave us valid JWTs, and leave it up to the issuer to verify these.
                    let header = jwt
                        .dangerous_parse_header_unverified()
                        .map_err(WalletIssuanceError::JwtParse)?;

                    let pubkey = header
                        .public_key()
                        .map_err(|e| WalletIssuanceError::VerifyingKeyFromPrivateKey(e.into()))?;
                    let cred_request = draft::CredentialRequest {
                        credential_type: credential_request_type.into(),
                        proof: Some(proof),
                    };

                    Ok::<_, WalletIssuanceError>(((pubkey, id), cred_request))
                }),
        )
        .await?
        .into_iter()
        .unzip();

        // The following two unwraps are safe because N > 0, see above.
        let responses = match credential_requests.len() {
            1 => {
                let credential_request = credential_requests.pop().unwrap();
                vec![
                    self.request_credential(credential_endpoint_url, &credential_request)
                        .await?,
                ]
            }
            _ => {
                let credential_requests = VecNonEmpty::try_from(credential_requests).unwrap();
                self.request_batch_credentials(credential_endpoint_url, credential_requests)
                    .await?
            }
        };
        let mut responses_and_pubkeys: VecDeque<_> = responses.into_iter().zip(pubkeys).collect();

        let docs = self
            .session_state
            .credential_previews
            .iter()
            // TODO (PVW-5554): reduce code duplication in the format arms
            .map(|preview| {
                let copy_count = usize::from(self.session_state.batch_size.get());

                // Get type metadata of attestation type
                let Some(type_metadata) = self
                    .session_state
                    .type_metadata
                    .get(&preview.credential_payload.attestation_type)
                else {
                    Err(WalletIssuanceError::TypeMetadataNotFound(
                        preview.credential_payload.attestation_type.clone(),
                    ))?
                };

                // Consume the amount of copies from the front of `responses_and_keys`.
                let copies = match preview.format {
                    Format::MsoMdoc => IssuedCredentialCopies::Mdoc(
                        responses_and_pubkeys
                            .drain(..copy_count)
                            .map(|(cred_response, (pubkey, key_id))| {
                                let credentials = cred_response
                                    .into_immediate_credentials()
                                    .ok_or(WalletIssuanceError::DeferredIssuanceUnsupported)?;

                                credentials.into_single_issued_mdoc(
                                    key_id,
                                    &pubkey,
                                    preview,
                                    &type_metadata.normalized_metadata,
                                    trust_anchors,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .try_into()
                            .expect("the resulting vector is never empty since 'copies' is nonzero"),
                    ),
                    Format::SdJwt => IssuedCredentialCopies::SdJwt(
                        responses_and_pubkeys
                            .drain(..copy_count)
                            .map(|(cred_response, (pubkey, key_id))| {
                                let credentials = cred_response
                                    .into_immediate_credentials()
                                    .ok_or(WalletIssuanceError::DeferredIssuanceUnsupported)?;

                                credentials.into_single_issued_sd_jwt(
                                    key_id,
                                    &pubkey,
                                    preview,
                                    &type_metadata.normalized_metadata,
                                    trust_anchors,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .try_into()
                            .expect("the resulting vector is never empty since 'copy_count' is nonzero"),
                    ),
                };

                // Verify that each of the resulting credentials contain exactly the same metadata integrity digest.
                let unique_integrities: HashSet<_> = match &copies {
                    IssuedCredentialCopies::Mdoc(mdocs) => mdocs
                        .iter()
                        .map(|mdoc| mdoc.type_metadata_integrity().map_err(WalletIssuanceError::Metadata))
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

                // Check that the integrity hash received in the credential matches
                // that of encoded JSON of the first metadata document.
                let verified_metadata = type_metadata.raw_metadata.clone().into_verified(integrity.clone())?;

                Ok::<_, WalletIssuanceError>(CredentialWithMetadata::new(
                    copies,
                    preview.credential_payload.attestation_type.clone(),
                    preview.credential_payload.expires,
                    preview.credential_payload.not_before,
                    type_metadata.normalized_metadata.extended_vcts(),
                    verified_metadata,
                ))
            })
            .try_collect()?;

        Ok(docs)
    }

    async fn reject_issuance(&self) -> Result<(), WalletIssuanceError> {
        let url = self
            .session_state
            .issuer_endpoints
            .batch_credential_endpoint
            .as_ref()
            .ok_or(WalletIssuanceError::NoBatchCredentialEndpoint)?
            .as_url();

        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::DELETE)?;

        self.message_client
            .reject(url, &dpop_header, &access_token_header)
            .await?;

        Ok(())
    }

    fn credential_previews(&self) -> &VecNonEmpty<CredentialPreview> {
        &self.session_state.credential_previews
    }

    fn type_metadata(&self) -> &HashMap<String, IssuanceTypeMetadata> {
        &self.session_state.type_metadata
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        &self.session_state.issuer_registration
    }
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    async fn request_credential(
        &self,
        url: &Url,
        credential_request: &draft::CredentialRequest,
    ) -> Result<CredentialResponse, WalletIssuanceError> {
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::POST)?;

        let response = self
            .message_client
            .request_credential(url, credential_request, &dpop_header, &access_token_header)
            .await?;

        Ok(response)
    }

    async fn request_batch_credentials(
        &self,
        url: &Url,
        credential_requests: VecNonEmpty<draft::CredentialRequest>,
    ) -> Result<Vec<CredentialResponse>, WalletIssuanceError> {
        let (dpop_header, access_token_header) = self.session_state.auth_headers(url.clone(), &Method::POST)?;

        let expected_response_count = credential_requests.len().get();
        let responses = self
            .message_client
            .request_credentials(
                url,
                &draft::CredentialRequests { credential_requests },
                &dpop_header,
                &access_token_header,
            )
            .await?;

        // The server must have responded with enough credential responses, N, so that the caller has exactly enough
        // responses for all copies of all credentials constructed.
        if responses.credential_responses.len() != expected_response_count {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseCount {
                found: responses.credential_responses.len(),
                expected: expected_response_count,
            });
        }

        Ok(responses.credential_responses)
    }
}

impl Credentials {
    /// Create an mdoc out of the credential response. Also verifies the credential.
    fn into_single_issued_mdoc(
        self,
        key_identifier: String,
        public_key: &PublicKey,
        preview: &CredentialPreview,
        normalized_type_metadata: &NormalizedTypeMetadata,
        trust_anchors: &TrustAnchors,
    ) -> Result<Mdoc, WalletIssuanceError> {
        match self {
            Self::MsoMdoc(mdoc_credentials) => {
                let MdocCredential {
                    credential: issuer_signed,
                } = mdoc_credentials.into_first();

                // Calculate the minimum of all the lengths of the random bytes
                // included in the attributes of `IssuerSigned`. If this value
                // is too low, we should not accept the attributes.
                if let Some(min) = issuer_signed.name_spaces.as_ref().and_then(|namespaces| {
                    namespaces
                        .as_ref()
                        .values()
                        .flat_map(|attributes| attributes.as_ref().iter().map(|TaggedBytes(item)| item.random.len()))
                        .min()
                }) && min < ATTR_RANDOM_LENGTH
                {
                    return Err(WalletIssuanceError::AttributeRandomLength(min, ATTR_RANDOM_LENGTH));
                }

                let credential_issuer_certificate = issuer_signed
                    .issuer_auth
                    .x5chain()
                    .map_err(WalletIssuanceError::IssuerCertificate)?
                    .into_first();

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new(key_identifier, issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(WalletIssuanceError::MdocVerification)?;

                let issued_credential_payload = CredentialPayload::from_mdoc(mdoc.clone(), normalized_type_metadata)?;

                Self::validate_credential(
                    preview,
                    public_key,
                    issued_credential_payload,
                    &credential_issuer_certificate,
                )?;

                Ok(mdoc)
            }
            Self::SdJwt(_) => Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: preview.format,
                actual: self,
            }),
        }
    }

    /// Create a credential out of the credential response. Also verifies the credential.
    fn into_single_issued_sd_jwt(
        self,
        key_identifier: String,
        holder_pubkey: &PublicKey,
        preview: &CredentialPreview,
        normalized_type_metadata: &NormalizedTypeMetadata,
        trust_anchors: &TrustAnchors,
    ) -> Result<SdJwtCopy, WalletIssuanceError> {
        match self {
            Self::MsoMdoc(_) => Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: preview.format,
                actual: self,
            }),
            Self::SdJwt(sd_jwt_credentials) => {
                let SdJwtCredential {
                    credential: unverified_sd_jwt,
                } = sd_jwt_credentials.into_first();

                let sd_jwt = unverified_sd_jwt.into_verified_against_trust_anchors(trust_anchors, &TimeGenerator)?;
                let issued_credential_payload = CredentialPayload::from_sd_jwt(sd_jwt.clone())?;

                // Store claim paths to later use in validation of selective disclosability of claims.
                // This prevents cloning `issued_credential_payload`.
                let issued_claims = issued_credential_payload
                    .previewable_payload
                    .attributes
                    .claim_paths(AttributesTraversalBehaviour::OnlyLeaves);

                Self::validate_credential(
                    preview,
                    holder_pubkey,
                    issued_credential_payload,
                    sd_jwt.issuer_leaf_certificate(),
                )?;

                // Verify whether each claims selective disclosability matches the metadata.
                // This validation is SD-JWT specific, and therefore cannot be part of `validate_credential`.
                Self::verify_selective_disclosability(&sd_jwt, issued_claims, normalized_type_metadata.clone())?;

                Ok(SdJwtCopy { key_identifier, sd_jwt })
            }
        }
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

impl IssuanceState {
    fn auth_headers(&self, url: Url, method: &Method) -> Result<(String, String), WalletIssuanceError> {
        let dpop_header = Dpop::new(
            &self.dpop_signing_key,
            url,
            method,
            Some(&self.access_token),
            self.dpop_nonce.clone(),
        )?;

        let access_token_header = "DPoP ".to_string() + self.access_token.as_ref();

        Ok((dpop_header.to_string(), access_token_header))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::num::NonZeroU8;
    use std::sync::Arc;
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
    use crypto::trust_anchor::BorrowingTrustAnchor;
    use crypto::utils::random_string;
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
            &oauth_metadata.token_endpoint,
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

    #[derive(Debug, Clone, Copy)]
    enum TokenResponseFields {
        AuthorizationDetails,
        Scope,
        Both,
        Neither,
    }

    fn test_start_issuance(
        ca: &Ca,
        trust_anchors: &TrustAnchors,
        issuer_metadata: IssuerMetadata,
        preview_payloads: Vec<(CredentialConfigurationId, Format, PreviewableCredentialPayload)>,
        type_metadata: TypeMetadata,
        token_response_fields: TokenResponseFields,
    ) -> Result<HttpIssuanceSession<MockVcMessageClient>, WalletIssuanceError> {
        let issuance_key = generate_pid_issuer_mock_with_registration(ca, &IssuerRegistration::new_mock()).unwrap();

        let scope = match token_response_fields {
            TokenResponseFields::Scope | TokenResponseFields::Both => Some(
                preview_payloads
                    .iter()
                    // Assume that the Credential Configuration's scope is its identifier with a `_scope` suffix.
                    .map(|(config_id, _, _)| format!("{config_id}_scope").parse().unwrap())
                    .collect::<HashSet<_>>(),
            ),
            TokenResponseFields::AuthorizationDetails | TokenResponseFields::Neither => None,
        };

        let authorization_details = match token_response_fields {
            TokenResponseFields::AuthorizationDetails | TokenResponseFields::Both => {
                let credential_ids_and_identifiers = VecNonEmpty::try_from(
                    preview_payloads
                        .iter()
                        .map(|(config_id, _, _)| (config_id, random_string(16)))
                        .collect_vec(),
                )
                .unwrap();

                Some(AuthorizationDetails::from_credential_ids_and_identifiers(
                    credential_ids_and_identifiers,
                ))
            }
            TokenResponseFields::Scope | TokenResponseFields::Neither => None,
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
        mock_msg_client
            .expect_request_type_metadata()
            .return_once(move |_url| Ok(TypeMetadataDocuments::from_single_example(type_metadata).2));

        mock_msg_client
            .expect_request_credential_preview()
            .return_once(move |_url, _access_token| {
                let previews = preview_payloads
                    .into_iter()
                    .map(|(config_id, format, preview_payload)| CredentialPreview {
                        config_id,
                        format,
                        credential_payload: preview_payload,
                        issuer_certificate: issuance_key.certificate().clone(),
                    })
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
            &oauth_metadata.token_endpoint,
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
    #[case::authorization_details(TokenResponseFields::AuthorizationDetails, false)]
    #[case::authorization_details_extra_configs(TokenResponseFields::AuthorizationDetails, true)]
    #[case::scope(TokenResponseFields::Scope, false)]
    #[case::scope_extra_configs(TokenResponseFields::Scope, true)]
    #[case::authorization_details_and_scope(TokenResponseFields::Both, false)]
    #[case::authorization_details_and_scope_extra_configs(TokenResponseFields::Both, true)]
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
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            token_response_fields,
        )
        .expect("starting issuance session should succeed");

        let preview = &session.credential_previews()[0];
        let type_metadata = session.type_metadata();
        assert_matches!(
                &preview.credential_payload.attributes.as_ref()["family_name"],
                Attribute::Single(AttributeValue::Text(v)) if v == "De Bruijn");

        assert_eq!(
            type_metadata
                .get(&preview.credential_payload.attestation_type)
                .unwrap()
                .normalized_metadata,
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
                CredentialConfigurationId::from("unknown_config_id".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            TokenResponseFields::AuthorizationDetails,
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
            vec![],
            TypeMetadata::pid_example(),
            TokenResponseFields::Scope,
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
                CredentialConfigurationId::from("unknown_config_id".to_string()),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            TokenResponseFields::Scope,
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
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            TokenResponseFields::Neither,
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
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::example_empty(PID_ATTESTATION_TYPE, &MockTimeGenerator::default()),
            )],
            TypeMetadata::empty_example_with_attestation_type("other_attestation_type"),
            TokenResponseFields::Neither,
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
                config_id.clone(),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            TokenResponseFields::Neither,
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
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            TokenResponseFields::Neither,
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
                    pid_config_id,
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    address_config_id,
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_address_example(&MockTimeGenerator::default()),
                ),
            ],
            TypeMetadata::pid_example(),
            TokenResponseFields::Neither,
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

        let config_id_mdoc: CredentialConfigurationId = "config_id_mdoc".to_string().into();
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
            (&config_id_mdoc, random_string(16)),
            (&config_id_sd_jwt, random_string(16))
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
                        config_id: config_id_mdoc.clone(),
                        format: Format::MsoMdoc,
                        credential_payload: preview_payload.clone(),
                        issuer_certificate: issuance_key.certificate().clone(),
                    },
                    CredentialPreview {
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
            &oauth_metadata.token_endpoint,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(oauth_metadata.challenge_endpoint.unwrap()),
            TokenRequest::new_mock(),
            &MockWiaClient::new(),
            &oauth_metadata.issuer,
            &TrustAnchors::from(&ca),
        )
        .now_or_never()
        .unwrap()
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::DifferentIssuers(_));
    }

    /// Return a new session ready for `accept_issuance()`.
    fn new_session_state(
        credential_previews: VecNonEmpty<CredentialPreview>,
        attestation_type: &str,
        issuance_type_metadata: IssuanceTypeMetadata,
        has_nonce_endpoint: bool,
    ) -> IssuanceState {
        let credential_request_types = credential_request_types_from_preview(&credential_previews, NonZeroU8::MIN);
        let issuer_identifier = "https://issuer.example.com".parse().unwrap();

        let config_id = credential_previews.first().config_id.clone();
        let mut issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier,
            vec![(
                config_id,
                CredentialKind::new(Format::SdJwt, attestation_type.to_string()),
            )],
        );
        issuer_metadata.batch_credential_issuance = None;
        if !has_nonce_endpoint {
            issuer_metadata.endpoints.nonce_endpoint = None;
        }

        IssuanceState {
            access_token: "access_token".to_string().into(),
            credential_issuer: issuer_metadata.credential_issuer,
            issuer_endpoints: issuer_metadata.endpoints,
            batch_size: NonZeroU8::MIN,
            credential_previews,
            credential_request_types,
            type_metadata: [(attestation_type.to_string(), issuance_type_metadata)].into(),
            issuer_registration: IssuerRegistration::new_mock(),
            dpop_signing_key: SigningKey::generate(),
            dpop_nonce: Some("dpop_nonce".to_string()),
        }
    }

    fn mock_openid_message_client_nonce(has_dpop_nonce: bool) -> MockVcMessageClient {
        let mut mock_msg_client = MockVcMessageClient::new();

        mock_msg_client
            .expect_request_nonce()
            .times(1)
            .with(eq(Url::parse("https://issuer.example.com/issuance/nonce").unwrap()))
            .return_once(move |_| {
                Ok((
                    NonceResponse {
                        c_nonce: Nonce::from("c_nonce".to_string()),
                    },
                    has_dpop_nonce.then(|| "new_dpop_nonce".to_string()),
                ))
            });

        mock_msg_client
    }

    #[derive(Debug, Clone)]
    struct MockCredentialSigner {
        pub trust_anchor: BorrowingTrustAnchor,
        issuer_key: Arc<KeyPair>,
        metadata_integrity: Integrity,
        previewable_payload: PreviewableCredentialPayload,
        status: StatusClaim,
    }

    impl MockCredentialSigner {
        pub fn new_with_preview_and_type_metadata_state() -> (Self, CredentialPreview, String, IssuanceTypeMetadata) {
            let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());
            let type_metadata = TypeMetadata::example_with_claim_name(&preview_payload.attestation_type, "family_name");

            Self::from_metadata_and_payload_with_preview_data(type_metadata, preview_payload)
        }

        pub fn from_metadata_and_payload_with_preview_data(
            type_metadata: TypeMetadata,
            preview_payload: PreviewableCredentialPayload,
        ) -> (Self, CredentialPreview, String, IssuanceTypeMetadata) {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let trust_anchor = ca.to_borrowing_trust_anchor();

            let issuer_registration = IssuerRegistration::new_mock();
            let issuer_key = generate_pid_issuer_mock_with_registration(&ca, &issuer_registration).unwrap();
            let issuer_certificate = issuer_key.certificate().clone();

            let (attestation_type, metadata_integrity, metadata_documents) =
                TypeMetadataDocuments::from_single_example(type_metadata);
            let (normalized_metadata, raw_metadata) = metadata_documents.into_normalized(&attestation_type).unwrap();

            let signer = Self {
                trust_anchor,
                issuer_key: Arc::new(issuer_key),
                metadata_integrity,
                previewable_payload: preview_payload.clone(),
                status: StatusClaim::new_mock(),
            };

            let preview = CredentialPreview {
                config_id: "config_id".to_string().into(),
                format: Format::MsoMdoc,
                credential_payload: preview_payload,
                issuer_certificate,
            };
            let issuance_type_metadata = IssuanceTypeMetadata {
                normalized_metadata,
                raw_metadata,
            };

            (signer, preview, attestation_type, issuance_type_metadata)
        }

        pub fn into_response_from_request(self, request: &draft::CredentialRequest) -> CredentialResponse {
            let proof_jwt = match request.proof.as_ref().unwrap() {
                draft::CredentialRequestProof::Jwt { jwt } => jwt,
            };
            let holder_pubkey = jwk_to_public_key(&proof_jwt.dangerous_parse_header_unverified().unwrap().jwk).unwrap();

            self.into_response_from_holder_pubkey(&holder_pubkey)
        }

        pub fn into_response_from_holder_pubkey(self, holder_pubkey: &PublicKey) -> CredentialResponse {
            let credential_payload = CredentialPayload::from_previewable_credential_payload_unvalidated(
                self.previewable_payload,
                Utc::now(),
                holder_pubkey,
                self.metadata_integrity,
                self.status,
            )
            .unwrap();

            let (issuer_signed, _) = credential_payload
                .into_signed_mdoc(&self.issuer_key)
                .now_or_never()
                .unwrap()
                .unwrap();

            CredentialResponse::new_immediate(Credentials::new_single_mdoc(issuer_signed))
        }
    }

    /// Check consistency and validity of the input of the /(batch_)credential endpoints.
    fn check_credential_endpoint_input(
        url: &Url,
        dpop_signing_key: &SigningKey,
        dpop_nonce: &str,
        dpop_header: &str,
        access_token_header: &str,
    ) {
        assert_eq!(access_token_header, "DPoP access_token".to_string());

        dpop_header
            .parse::<Dpop>()
            .unwrap()
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

    #[rstest]
    fn test_accept_issuance(
        #[values(true, false)] multiple_creds: bool,
        #[values(
            TestNonceEndpoint::Absent,
            TestNonceEndpoint::Present,
            TestNonceEndpoint::PresentWithDpopNonce
        )]
        nonce_endpoint: TestNonceEndpoint,
    ) {
        let (signer, preview_data, attestation_type, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata_state();
        let trust_anchor = TrustAnchors::try_from(vec![signer.trust_anchor.clone()]).unwrap();
        let wscd = MockRemoteWscd::default();

        let (mut mock_msg_client, has_nonce_endpoint, expected_dpop_nonce) = match nonce_endpoint {
            TestNonceEndpoint::Absent => (MockVcMessageClient::new(), false, "dpop_nonce"),
            TestNonceEndpoint::Present => (mock_openid_message_client_nonce(false), true, "dpop_nonce"),
            TestNonceEndpoint::PresentWithDpopNonce => (mock_openid_message_client_nonce(true), true, "new_dpop_nonce"),
        };

        let session_state = new_session_state(
            if multiple_creds {
                vec_nonempty![preview_data.clone(), preview_data]
            } else {
                vec_nonempty![preview_data]
            },
            &attestation_type,
            type_metadata,
            has_nonce_endpoint,
        );

        // The client must use `request_credentials()` (which uses `/batch_credentials`) iff more than one credential
        // is being issued, and `request_credential()` instead (which uses `/credential`).
        if multiple_creds {
            mock_msg_client.expect_request_credentials().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_requests, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state.dpop_signing_key,
                        expected_dpop_nonce,
                        dpop_header,
                        access_token_header,
                    );

                    let credential_responses = credential_requests
                        .credential_requests
                        .iter()
                        .zip(std::iter::repeat_n(
                            signer,
                            credential_requests.credential_requests.len().get(),
                        ))
                        .map(|(request, signer)| signer.into_response_from_request(request))
                        .collect();

                    Ok(draft::CredentialResponses { credential_responses })
                }
            });
        } else {
            mock_msg_client.expect_request_credential().times(1).return_once({
                let session_state = session_state.clone();
                move |url, credential_request, dpop_header, access_token_header| {
                    check_credential_endpoint_input(
                        url,
                        &session_state.dpop_signing_key,
                        expected_dpop_nonce,
                        dpop_header,
                        access_token_header,
                    );

                    let response = signer.into_response_from_request(credential_request);

                    Ok(response)
                }
            });
        }

        let credential_copies = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        }
        .accept_issuance(&trust_anchor, &wscd)
        .now_or_never()
        .unwrap()
        .expect("accepting issuance should succeed");

        let expected_credential_count = if multiple_creds { 2 } else { 1 };
        assert_eq!(credential_copies.len(), expected_credential_count);
    }

    #[test]
    fn test_accept_issuance_wrong_response_count() {
        let (signer, preview_data, attestation_type, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata_state();
        let trust_anchor = TrustAnchors::try_from(vec![signer.trust_anchor.clone()]).unwrap();

        let mut mock_msg_client = mock_openid_message_client_nonce(false);

        mock_msg_client.expect_request_credentials().return_once(
            |_url, credential_requests, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_requests.credential_requests.first());
                // Return one credential response.
                let responses = draft::CredentialResponses {
                    credential_responses: vec![response],
                };

                Ok(responses)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(
                vec_nonempty![preview_data.clone(), preview_data],
                &attestation_type,
                type_metadata,
                true,
            ),
        }
        .accept_issuance(&trust_anchor, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::UnexpectedCredentialResponseCount { found: 1, expected: 2 }
        );
    }

    #[test]
    fn test_accept_issuance_incorrect_resource_integrity() {
        let (mut signer, preview_data, attestation_type, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata_state();
        let trust_anchor = TrustAnchors::try_from(vec![signer.trust_anchor.clone()]).unwrap();

        // Include a random resource integrity in the MSO of the returned mdoc.
        signer.metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));

        let mut mock_msg_client = mock_openid_message_client_nonce(false);

        mock_msg_client.expect_request_credential().return_once(
            |_url, credential_request, _dpop_header, _access_token_header| {
                let response = signer.into_response_from_request(credential_request);

                Ok(response)
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(vec_nonempty![preview_data], &attestation_type, type_metadata, true),
        }
        .accept_issuance(&trust_anchor, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(
            error,
            WalletIssuanceError::TypeMetadataVerification(TypeMetadataChainError::ResourceIntegrity(_))
        );
    }

    #[rstest]
    fn test_accept_issuance_deferred_issuance(#[values(false, true)] is_batch: bool) {
        let (signer, preview_data, attestation_type, type_metadata) =
            MockCredentialSigner::new_with_preview_and_type_metadata_state();
        let trust_anchor = TrustAnchors::try_from(vec![signer.trust_anchor.clone()]).unwrap();

        let mut mock_msg_client = mock_openid_message_client_nonce(false);

        let response = CredentialResponse::Deferred {
            transaction_id: "12345".to_string(),
            interval: Duration::from_hours(24),
        };

        let previews = if is_batch {
            mock_msg_client.expect_request_credentials().return_once(
                |_url, credential_requests, _dpop_header, _access_token_header| {
                    let responses = draft::CredentialResponses {
                        credential_responses: vec![response; credential_requests.credential_requests.len().get()],
                    };

                    Ok(responses)
                },
            );

            vec_nonempty![preview_data.clone(), preview_data]
        } else {
            mock_msg_client
                .expect_request_credential()
                .return_once(|_url, _credential_request, _dpop_header, _access_token_header| Ok(response));

            vec_nonempty![preview_data]
        };

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, &attestation_type, type_metadata, true),
        }
        .accept_issuance(&trust_anchor, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::DeferredIssuanceUnsupported);
    }

    fn mock_credential_response_credential() -> (
        Credentials,
        CredentialPreview,
        IssuanceTypeMetadata,
        PublicKey,
        TrustAnchors,
    ) {
        let (signer, preview_data, _, type_metadata) = MockCredentialSigner::new_with_preview_and_type_metadata_state();
        let trust_anchor = TrustAnchors::try_from(vec![signer.trust_anchor.clone()]).unwrap();
        let holder_pubkey = PublicKey::from(*SigningKey::generate().verifying_key());
        let credential_response = signer
            .into_response_from_holder_pubkey(&holder_pubkey)
            .into_immediate_credentials()
            .unwrap();

        (
            credential_response,
            preview_data,
            type_metadata,
            holder_pubkey,
            trust_anchor,
        )
    }

    #[test]
    fn test_credential_response_into_mdoc() {
        let (credentials, preview_data, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        let _issued_credential = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[test]
    fn test_credential_response_into_mdoc_public_key_mismatch_error() {
        let (credentials, preview_data, type_metadata, _, trust_anchor) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = PublicKey::from(*SigningKey::generate().verifying_key());
        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &other_public_key,
                &preview_data,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::PublicKeyMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_attribute_random_length_error() {
        let (credentials, preview_data, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

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
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH));
    }

    #[test]
    fn test_credential_response_into_mdoc_issuer_certificate_mismatch_error() {
        let (credentials, preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` using a different issuer
        // public key in the preview than is contained within the response should fail.
        let other_ca = Ca::generate_issuer_mock_ca().unwrap();
        let other_issuance_key =
            generate_pid_issuer_mock_with_registration(&other_ca, &IssuerRegistration::new_mock()).unwrap();
        let preview_data = CredentialPreview {
            issuer_certificate: other_issuance_key.certificate().clone(),
            ..preview
        };

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview_data,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuerMismatch);
    }

    #[test]
    fn test_credential_response_into_mdoc_mdoc_verification_error() {
        let (credentials, preview, type_metadata, holder_public_key, _) = mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &TrustAnchors::empty(),
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::MdocVerification(_));
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

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

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_issuer_mismatch_error() {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different `issuer_uri` in the preview than
        // contained within the response should fail.
        preview.credential_payload.issuer = "https://other-issuer.example.com".parse().unwrap();

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_doctype_mismatch_error() {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        preview.credential_payload.attestation_type = String::from("other.attestation_type");

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error() {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.

        preview.credential_payload.not_before = Some((Utc::now() + chrono::Duration::days(1)).into());

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attestation_qualification_mismatch_error() {
        let (credentials, mut preview, type_metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential();

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        preview.credential_payload.attestation_qualification = AttestationQualification::PubEAA;

        let error = credentials
            .into_single_issued_mdoc(
                "key_id".to_string(),
                &holder_public_key,
                &preview,
                &type_metadata.normalized_metadata,
                &trust_anchor,
            )
            .expect_err("should not be able to convert CredentialResponse into Mdoc");

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
