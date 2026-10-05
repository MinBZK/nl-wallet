use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::convert::identity;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;

use attestation_data::attributes::AttributesTraversalBehaviour;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::metadata::AttestationClaims;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use crypto::PublicKey;
use crypto::trust_anchor::TrustAnchors;
use derive_more::Debug;
use futures::TryFutureExt;
use futures::future::try_join_all;
use futures::try_join;
use http_utils::reqwest::HttpClient;
use itertools::Either;
use itertools::Itertools;
use jwt::nonce::Nonce;
use mdoc::ATTR_RANDOM_LENGTH;
use mdoc::holder::Mdoc;
use mdoc::utils::serialization::TaggedBytes;
use nonempty_collections::NEMap;
use nonempty_collections::NESet;
use oauth::dpop::DPOP_HEADER_NAME;
use oauth::dpop::DPOP_NONCE_HEADER_NAME;
use oauth::dpop::Dpop;
use oauth::dpop::DpopError;
use oauth::dpop::DpopNonce;
use oauth::errors::RemoteErrorCode;
use oauth::errors::RemoteErrorResponse;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::issuer_identifier::IssuerUrl;
use oauth::scope::Scope;
use oauth::token::AccessToken;
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
use wscd::issuance::IssuanceKeyResult;
use wscd::issuance::IssuanceWscd;
use wscd::payload::wia::WIA_HEADER_NAME;
use wscd::payload::wia::WIA_POP_HEADER_NAME;
use wscd::payload::wia::WiaDisclosure;
use wscd::wia::WiaClient;

use super::IssuanceSession;
use super::OfferedCredentialMetadata;
use super::OfferedCredentialPreview;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use super::credential::IssuedCredentialCopies;
use super::credential::IssuedCredentialMetadata;
use super::credential::MdocCopy;
use super::credential::SdJwtCopy;
use super::issuer_registration::IssuerRegistration;
use crate::authorization_details::CredentialId;
use crate::authorization_details::IssuerAuthorizationDetails;
use crate::client_auth::ClientAttestationChallengeMechanism;
use crate::client_auth::fetch_client_auth_challenge;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestIdentifier;
use crate::credential::CredentialResponse;
use crate::credential::Credentials;
use crate::credential::MdocCredential;
use crate::credential::SdJwtCredential;
use crate::errors::CredentialErrorCode;
use crate::errors::CredentialPreviewErrorCode;
use crate::errors::VciTokenErrorCode;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::CredentialMetadata;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::nonce::response::NonceResponse;
use crate::preview::CredentialPreviewResponse;
use crate::token::CredentialPreview;
use crate::token::TokenRequestGrantType;
use crate::token::VciTokenRequest;
use crate::token::VciTokenResponse;

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
        token_request: &VciTokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(VciTokenResponse, Option<DpopNonce>), WalletIssuanceError>;

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
        token_request: &VciTokenRequest,
        dpop_header: &Dpop,
        wia: &WiaDisclosure,
    ) -> Result<(VciTokenResponse, Option<DpopNonce>), WalletIssuanceError> {
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
                        .json::<RemoteErrorResponse<VciTokenErrorCode>>()
                        .await
                        .map_err(WalletIssuanceError::TokenRequestHttp)?;

                    Err(WalletIssuanceError::VciTokenRequest(Box::new(error)))
                } else {
                    let dpop_nonce = Self::dpop_nonce(&response)?;
                    let deserialized = response
                        .json::<VciTokenResponse>()
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
}

/// The parts of a [`CredentialConfiguration`] with a supported format that are relevant to an issuance session.
#[derive(Debug)]
struct SupportedConfiguration {
    credential_kind: CredentialKind,
    credential_metadata: Option<CredentialMetadata>,
    type_metadata_uri: Option<IssuerUrl>,
}

impl SupportedConfiguration {
    /// Returns `None` if the format of the [`CredentialConfiguration`] is not supported.
    fn try_new(config: CredentialConfiguration) -> Option<Self> {
        let credential_kind = config.format.credential_kind()?;

        Some(Self {
            credential_kind,
            credential_metadata: config.credential_metadata,
            type_metadata_uri: config.type_metadata_uri,
        })
    }
}

/// Internal helper type that represents the Credential Configurations from the Credential Offer that the issuer granted
/// to the holder in the Token Response. The [`CredentialConfiguration`]s are sourced from the Issuer Metadata.
#[derive(Debug)]
enum GrantedConfigurations {
    /// The result of a Token Response that did not contain `authorization_details`. Credentials may only be identified
    /// by their Credential Configuration Identifier when the holder calls the Credential Endpoint to fetch the
    /// credentials.
    WithoutIdentifiers(HashMap<CredentialConfigurationId, CredentialConfiguration>),

    /// The result of a Token Response that did contain `authorization_details`, where each Credential Configuration
    /// has at least one Credential Identifier of an offered credential. The holder then uses these Credential
    /// Identifiers when fetching the credentials using the Credential Endpoint.
    ///
    /// The Credential Identifiers are unique across Credential Configurations, as this is required in order to identify
    /// credentials at the Credential Endpoint.
    WithIdentifiers(HashMap<CredentialConfigurationId, (CredentialConfiguration, HashSet<CredentialId>)>),
}

/// The [`GrantedConfigurations`] that have a supported format, see [`GrantedConfigurations::into_supported`]. There is
/// always at least one Credential Configuration or at least one Credential Identifier per Credential Configuration.
#[derive(Debug)]
enum SupportedConfigurations {
    /// See [`GrantedConfigurations::WithoutIdentifiers`].
    WithoutIdentifiers(NEMap<CredentialConfigurationId, SupportedConfiguration>),

    /// See [`GrantedConfigurations::WithIdentifiers`].
    WithIdentifiers(NEMap<CredentialConfigurationId, (SupportedConfiguration, NESet<CredentialId>)>),
}

impl GrantedConfigurations {
    /// Create [`GrantedConfigurations`] by combining the Credential Configurations that were present in the Credential
    /// Offer with the `scope` and `authorization_details` fields as received in the Token Response, discarding any
    /// Credential Configurations that the issuer no longer offers.
    fn new_from_token_response(
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        scope: Option<&HashSet<Scope>>,
        authorization_details: Option<IssuerAuthorizationDetails>,
    ) -> Result<Self, WalletIssuanceError> {
        match (scope, authorization_details) {
            // If the Token Response contained `authorization_details`, use that and ignore any `scope` values. Returns
            // an error if any Credential Configuration ID was not present in the Credential Offer.
            (_, Some(authorization_details)) => {
                Self::new_from_authorization_details(credential_configurations, authorization_details)
            }

            // If the Token Response contained `scope` values, select only those Credential Configurations that have
            // this scope. Returns an error if no scope values were provided or if any of the scope values do not refer
            // to Credential Configurations present in the Credential Offer.
            (Some(scope), None) => Self::new_from_scope(credential_configurations, scope),

            // If neither the `authorization_details` nor the `scope` field was present in the Token Response, it means
            // that the issuer offers all of the Credential Configurations from the Credential Offer.
            (None, None) => Ok(Self::WithoutIdentifiers(credential_configurations)),
        }
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the
    /// `authorization_details` field from the Token Response.
    fn new_from_authorization_details(
        mut credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        authorization_details: IssuerAuthorizationDetails,
    ) -> Result<Self, WalletIssuanceError> {
        // Pair each Credential Configuration Identifier with a known Credential Configuration fetched from the Issuer
        // Metadata based on the Credential Offer. Return an error if any of the identifiers were not part of the offer.
        let (offered_configs, unknown_config_ids): (HashMap<_, _>, Vec<_>) = authorization_details
            .into_credential_ids_by_config_ids()
            .into_iter()
            .partition_map(
                |(config_id, credential_ids)| match credential_configurations.remove(&config_id) {
                    Some(config) => Either::Left((config_id, (config, credential_ids))),
                    None => Either::Right(config_id),
                },
            );

        if !unknown_config_ids.is_empty() {
            return Err(WalletIssuanceError::AuthorizationDetailsUnknownCredentialConfigIds(
                unknown_config_ids,
            ));
        }

        // Finally, check that all of the Credential Identifiers are unique amongst the Credential Configurations, as
        // this is required for identifying credentials at the Credential Endpoint.
        let duplicate_credential_ids = offered_configs
            .values()
            .flat_map(|(_config, credential_ids)| credential_ids)
            .duplicates()
            .collect::<HashSet<_>>();

        if !duplicate_credential_ids.is_empty() {
            let duplicates = duplicate_credential_ids
                .into_iter()
                .map(|credential_id| {
                    let config_ids = offered_configs
                        .iter()
                        .filter_map(|(config_id, (_config, credential_ids))| {
                            credential_ids.contains(credential_id).then_some(config_id)
                        })
                        .cloned()
                        .collect();

                    (credential_id.clone(), config_ids)
                })
                .collect();

            return Err(WalletIssuanceError::AuthorizationDetailsDuplicateCredentialIds(
                duplicates,
            ));
        }

        Ok(Self::WithIdentifiers(offered_configs))
    }

    /// Filter the Credential Configurations that were present in the Credential Offer based on the `scope` field from
    /// the Token Response.
    fn new_from_scope(
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        scope: &HashSet<Scope>,
    ) -> Result<Self, WalletIssuanceError> {
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

        Ok(Self::WithoutIdentifiers(offered_configs))
    }

    /// Discard the Credential Configurations with an unsupported format. Returns `None` if no Credential Configurations
    /// remain.
    fn into_supported(self) -> Option<SupportedConfigurations> {
        match self {
            Self::WithoutIdentifiers(configs) => NEMap::try_from_map(
                configs
                    .into_iter()
                    .filter_map(|(config_id, config)| Some((config_id, SupportedConfiguration::try_new(config)?)))
                    .collect::<HashMap<_, _>>(),
            )
            .map(SupportedConfigurations::WithoutIdentifiers),
            Self::WithIdentifiers(configs) => NEMap::try_from_map(
                configs
                    .into_iter()
                    .filter_map(|(config_id, (config, credential_ids))| {
                        // A Credential Configuration without any Credential Identifiers does not offer any credentials,
                        // so it can be discarded as well. Note that this does not occur in practice, as the
                        // `authorization_details` contain at least one Credential Identifier per configuration.
                        let credential_ids = NESet::try_from_set(credential_ids)?;

                        Some((config_id, (SupportedConfiguration::try_new(config)?, credential_ids)))
                    })
                    .collect::<HashMap<_, _>>(),
            )
            .map(SupportedConfigurations::WithIdentifiers),
        }
    }
}

impl SupportedConfigurations {
    fn credential_config_iter(&self) -> impl Iterator<Item = (&CredentialConfigurationId, &SupportedConfiguration)> {
        match self {
            Self::WithoutIdentifiers(credential_configs) => Either::Left(credential_configs.iter()),
            Self::WithIdentifiers(credential_configs) => Either::Right(
                credential_configs
                    .iter()
                    .map(|(config_id, (config, _ids))| (config_id, config)),
            ),
        }
    }

    /// Create an [`OfferedCredentials`] without previews, containing an entry for every offered Credential
    /// Configuration or, if applicable, every offered Credential Identifier.
    fn into_offered_credentials_without_previews(self) -> OfferedCredentials {
        let offered_credentials = match self {
            Self::WithoutIdentifiers(configs) => configs
                .into_nonempty_iter()
                .map(|(config_id, config)| {
                    OfferedCredential::new_by_config_id(config_id, config.credential_kind.format)
                })
                .collect(),
            Self::WithIdentifiers(configs) => configs
                .into_nonempty_iter()
                .flat_map(|(config_id, (config, credential_ids))| {
                    let format = config.credential_kind.format;

                    credential_ids.into_nonempty_iter().map(move |credential_id| {
                        OfferedCredential::new_by_credential_id(credential_id, config_id.clone(), format)
                    })
                })
                .collect(),
        };

        OfferedCredentials::WithoutPreviews(offered_credentials)
    }

    /// Create an [`OfferedCredentials`] with previews, checking that the `CredentialPreview`s exactly match the
    /// credentials that were offered by the issuer. This returns an error when any previews are missing or when excess
    /// previews are received.
    fn into_offered_credentials_with_previews(
        self,
        credential_previews: VecNonEmpty<CredentialPreview>,
    ) -> Result<OfferedCredentials, WalletIssuanceError> {
        let (offered_credentials, excess_identifiers): (Vec<_>, Vec<_>) = match self {
            // If the offered credential configurations did not contain credential identifiers because the issuer did
            // not send `authorization_details`, match every preview against its `config_id` value only.
            Self::WithoutIdentifiers(configs) => {
                // Convert to a regular map so that matched configurations can be removed from it.
                let mut configs = HashMap::from(configs);

                let (offered_credentials, excess_identifiers) =
                    credential_previews.into_iter().partition_map(|preview| {
                        // If both the config_id and format match, remove the Credential Configuration. Otherwise,
                        // consider this Credential Preview an excess preview.
                        match configs.entry(preview.config_id.clone()) {
                            Entry::Occupied(occupied_entry)
                                if occupied_entry.get().credential_kind.format == preview.format =>
                            {
                                let (config_id, _config) = occupied_entry.remove_entry();
                                let offered_credential = OfferedCredential::new_by_config_id(config_id, preview.format);
                                Either::Left((offered_credential, preview.credential_payload))
                            }
                            _ => Either::Right((preview.config_id, preview.credential_id, preview.format)),
                        }
                    });

                // If there are any offered credential configurations remaining, the preview did not contain everything
                // that was offered.
                if !configs.is_empty() {
                    let missing = configs
                        .into_iter()
                        .map(|(config_id, config)| (config_id, None, config.credential_kind.format))
                        .collect();

                    return Err(WalletIssuanceError::PreviewMissingCredentials(missing));
                }

                (offered_credentials, excess_identifiers)
            }
            // If the issuer did send `authorization_details`, match every preview exactly against both its `config_id`
            // and `credential_id` values.
            Self::WithIdentifiers(configs) => {
                // Convert to regular collections so that matched credential identifiers can be removed from them.
                let mut configs = configs
                    .into_iter()
                    .map(|(config_id, (config, credential_ids))| (config_id, (config, HashSet::from(credential_ids))))
                    .collect::<HashMap<_, _>>();

                let (offered_credentials, excess_identifiers) =
                    credential_previews.into_iter().partition_map(|preview| {
                        // If the config_id, credential_id and format all match, remove the credential_id. Otherwise,
                        // consider this Credential Preview an excess preview.
                        match configs
                            .get_mut(&preview.config_id)
                            .and_then(|(config, credential_ids)| {
                                if config.credential_kind.format == preview.format {
                                    credential_ids.take(&preview.credential_id)
                                } else {
                                    None
                                }
                            }) {
                            Some(credential_id) => {
                                let offered_credential = OfferedCredential::new_by_credential_id(
                                    credential_id,
                                    preview.config_id.clone(),
                                    preview.format,
                                );

                                Either::Left((offered_credential, preview.credential_payload))
                            }
                            None => Either::Right((preview.config_id, preview.credential_id, preview.format)),
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
                        .flat_map(|(config_id, (config, credential_ids))| {
                            let credential_id_count = credential_ids.len();
                            let format = config.credential_kind.format;
                            std::iter::repeat_n(config_id, credential_id_count)
                                .zip(credential_ids)
                                .map(move |(config_id, credential_id)| (config_id, Some(credential_id), format))
                        })
                        .collect();

                    return Err(WalletIssuanceError::PreviewMissingCredentials(missing));
                }

                (offered_credentials, excess_identifiers)
            }
        };

        // If any of the previews could not be resolved against what was offered, report this as an error. As there is
        // at least one preview, having no offered credentials implies that every preview was in excess.
        match (
            VecNonEmpty::try_from(offered_credentials),
            excess_identifiers.is_empty(),
        ) {
            (Ok(offered_credentials), true) => Ok(OfferedCredentials::WithPreviews(offered_credentials)),
            _ => Err(WalletIssuanceError::PreviewExcessCredentials(excess_identifiers)),
        }
    }
}

#[derive(Debug)]
struct IssuanceState {
    access_token: AccessToken,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    batch_size: NonZeroU8,
    // Keep metadata separate from offered credentials to prevent duplication. Metadata is per credential
    // configuration, not per credential. `HttpIssuanceSession::create` enforces that every credential configuration
    // has metadata.
    metadata: HashMap<CredentialConfigurationId, OfferedCredentialMetadata>,
    offered_credentials: OfferedCredentials,
    issuer_registration: IssuerRegistration,
    #[debug(skip)]
    dpop_signing_key: SigningKey,
    dpop_nonce: Option<DpopNonce>,
}

#[derive(Debug)]
enum OfferedCredentials {
    /// The issuer provided a preview for each offered credential. Maintains the order as received from the Credential
    /// Preview endpoint.
    WithPreviews(VecNonEmpty<(OfferedCredential, PreviewableCredentialPayload)>),

    /// The issuer does not have a Credential Preview endpoint. The offered credentials have no meaningful order.
    WithoutPreviews(NESet<OfferedCredential>),
}

impl OfferedCredentials {
    fn len(&self) -> NonZeroUsize {
        match self {
            OfferedCredentials::WithPreviews(creds) => creds.len(),
            OfferedCredentials::WithoutPreviews(creds) => creds.len(),
        }
    }

    /// Iterate over the offered credentials and optional previews.
    fn iter(&self) -> impl Iterator<Item = (&OfferedCredential, Option<&PreviewableCredentialPayload>)> {
        match self {
            OfferedCredentials::WithPreviews(creds) => Either::Left(
                creds
                    .iter()
                    .map(|(offered_credential, preview)| (offered_credential, Some(preview))),
            ),
            OfferedCredentials::WithoutPreviews(creds) => {
                Either::Right(creds.iter().map(|offered_credential| (offered_credential, None)))
            }
        }
    }
}

/// A single credential offered by the issuer.
#[derive(Debug, PartialEq, Eq, Hash)]
struct OfferedCredential {
    /// Present when the Token Response contained `authorization_details`.
    credential_id: Option<CredentialId>,

    /// Used to look up the metadata of this credential.
    config_id: CredentialConfigurationId,

    /// As specified by the Credential Configuration.
    format: Format,
}

impl OfferedCredential {
    /// The Token Response did not contain `authorization_details`, so the credential is requested by its Credential
    /// Configuration Identifier.
    fn new_by_config_id(config_id: CredentialConfigurationId, format: Format) -> Self {
        Self {
            credential_id: None,
            config_id,
            format,
        }
    }

    /// The Token Response contained `authorization_details`, so the credential is requested by its Credential
    /// Identifier.
    fn new_by_credential_id(credential_id: CredentialId, config_id: CredentialConfigurationId, format: Format) -> Self {
        Self {
            credential_id: Some(credential_id),
            config_id,
            format,
        }
    }

    fn to_request_identifier(&self) -> CredentialRequestIdentifier {
        match &self.credential_id {
            Some(credential_id) => CredentialRequestIdentifier::CredentialIdentifier(credential_id.clone()),
            None => CredentialRequestIdentifier::CredentialConfigurationId(self.config_id.clone()),
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
fn map_pre_authorized_token_error(error: WalletIssuanceError, token_request: &VciTokenRequest) -> WalletIssuanceError {
    let is_pre_authorized = matches!(
        token_request.oauth_request.grant_type,
        TokenRequestGrantType::PreAuthorizedCode { .. }
    );

    match &error {
        WalletIssuanceError::VciTokenRequest(response)
            if is_pre_authorized && response.error == RemoteErrorCode::Known(VciTokenErrorCode::InvalidGrant) =>
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
        issuer_registration: IssuerRegistration,
        batch_size: NonZeroU8,
        token_endpoint: Url,
        client_auth_challenge: ClientAttestationChallengeMechanism,
        token_request: VciTokenRequest,
        wia_client: &impl WiaClient,
        auth_server_identifier: &IssuerIdentifier,
    ) -> Result<Self, WalletIssuanceError> {
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

        let offered_configurations = GrantedConfigurations::new_from_token_response(
            credential_configurations,
            token_response.oauth_response.scope.as_ref(),
            token_response.authorization_details,
        )?
        .into_supported()
        .ok_or(WalletIssuanceError::NoSupportedCredentialConfigurations)?;

        let (metadata, offered_credentials) = match issuer_endpoints.credential_preview_endpoint.as_ref() {
            Some(preview_endpoint) => {
                let (metadata, credential_previews) = try_join!(
                    Self::fetch_metadata(
                        offered_configurations.credential_config_iter(),
                        &credential_issuer,
                        &message_client
                    ),
                    Self::request_previews(
                        preview_endpoint.as_url().clone(),
                        &token_response.oauth_response.access_token,
                        &message_client
                    )
                )?;

                (
                    metadata,
                    offered_configurations.into_offered_credentials_with_previews(credential_previews)?,
                )
            }
            None => {
                let metadata = Self::fetch_metadata(
                    offered_configurations.credential_config_iter(),
                    &credential_issuer,
                    &message_client,
                )
                .await?;

                (
                    metadata,
                    offered_configurations.into_offered_credentials_without_previews(),
                )
            }
        };

        let session_state = IssuanceState {
            access_token: token_response.oauth_response.access_token,
            credential_issuer,
            issuer_endpoints,
            batch_size,
            offered_credentials,
            metadata,
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

    async fn request_previews(
        preview_endpoint: Url,
        access_token: &AccessToken,
        message_client: &H,
    ) -> Result<VecNonEmpty<CredentialPreview>, WalletIssuanceError> {
        let CredentialPreviewResponse { credential_previews } = message_client
            .request_credential_preview(preview_endpoint, access_token)
            .await?;

        Ok(credential_previews)
    }

    // Determine how each offered configuration's metadata is obtained. An SD-JWT that carries a `type_metadata_uri`
    // is described by the remotely fetched SD-JWT VC Type Metadata. Every other configuration is described by the
    // Credential Metadata in the Credential Issuer metadata: an SD-JWT without a `type_metadata_uri` and mdocs. Any
    // `type_metadata_uri` on an mdoc is ignored. Missing metadata is an error.
    async fn fetch_metadata(
        credential_configurations: impl IntoIterator<Item = (&CredentialConfigurationId, &SupportedConfiguration)>,
        credential_issuer: &IssuerIdentifier,
        message_client: &H,
    ) -> Result<HashMap<CredentialConfigurationId, OfferedCredentialMetadata>, WalletIssuanceError> {
        let mut type_metadata_configs = Vec::new();
        let mut credential_metadata = HashMap::new();
        let mut missing_metadata_config_ids = Vec::new();

        for (config_id, config) in credential_configurations {
            match (
                config.credential_kind.format,
                config.type_metadata_uri.as_ref(),
                config.credential_metadata.as_ref(),
            ) {
                (Format::SdJwt, Some(uri), _) => {
                    type_metadata_configs.push((uri, (config.credential_kind.attestation_type.as_str(), config_id)));
                }
                (Format::SdJwt | Format::MsoMdoc, _, Some(metadata)) => {
                    credential_metadata.insert(
                        config_id.clone(),
                        OfferedCredentialMetadata::CredentialMetadata(metadata.clone()),
                    );
                }
                (Format::SdJwt | Format::MsoMdoc, _, None) => missing_metadata_config_ids.push(config_id.clone()),
            }
        }

        if !missing_metadata_config_ids.is_empty() {
            return Err(WalletIssuanceError::MetadataMissing(missing_metadata_config_ids));
        }

        // Group credential configurations by their type metadata URI so that each URI is fetched only once.
        let configs_per_uri = type_metadata_configs.into_iter().into_group_map();

        // Check that all URIs have the same scheme and host as the Issuer Identifier, as is required by our profile.
        let mismatched_uris = configs_per_uri
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
        let (uris_and_vcts_with_configs, multi_vct_uris): (Vec<_>, Vec<_>) =
            configs_per_uri.into_iter().partition_map(|(uri, vct_and_config_ids)| {
                let (vcts, config_ids): (Vec<_>, Vec<_>) = vct_and_config_ids.into_iter().unzip();

                match vcts.into_iter().unique().exactly_one() {
                    Ok(vct) => Either::Left((uri, vct, config_ids)),
                    Err(vcts_iter) => {
                        let vcts = vcts_iter.map(str::to_string).collect_vec();

                        Either::Right((uri.clone(), vcts))
                    }
                }
            });

        if !multi_vct_uris.is_empty() {
            return Err(WalletIssuanceError::TypeMetadataUriMultipleVcts(Box::new(
                multi_vct_uris,
            )));
        }

        // Fetch type metadata documents from URIs, then normalize the chain of documents.
        let metadata_per_config_id = try_join_all(uris_and_vcts_with_configs.into_iter().map(
            async |(uri, vct, config_ids)| -> Result<_, WalletIssuanceError> {
                let documents = message_client.request_type_metadata(uri.as_url().clone()).await?;

                let (normalized, raw) = documents
                    .clone()
                    .into_normalized(vct)
                    .map_err(WalletIssuanceError::TypeMetadataVerification)?;
                let metadata = OfferedCredentialMetadata::TypeMetadata { normalized, raw };

                // Duplicate the resulting metadata per Credential Configuration ID.
                let config_id_count = config_ids.len();
                let config_ids_and_metadata = config_ids
                    .into_iter()
                    .cloned()
                    .zip(std::iter::repeat_n(metadata, config_id_count))
                    .collect_vec();

                Ok(config_ids_and_metadata)
            },
        ))
        .await?
        .into_iter()
        .flatten()
        .chain(credential_metadata)
        .collect();

        Ok(metadata_per_config_id)
    }

    async fn fetch_credential(
        &self,
        offered_credential: &OfferedCredential,
        credential_preview: Option<&PreviewableCredentialPayload>,
        keys: VecNonEmpty<IssuanceKeyResult>,
        dpop_nonce: Option<DpopNonce>,
        trust_anchors: &TrustAnchors,
    ) -> Result<CredentialWithMetadata, WalletIssuanceError> {
        // Extract pairs of key identifiers and public keys and proofs from the WSCD response. Note that the WSCD may
        // have returned fewer proofs than we requested, which means the issuer will provide us with fewer credential
        // copies.
        let (key_ids_and_public_keys, proofs): (VecNonEmpty<_>, _) = keys
            .into_nonempty_iter()
            .map(|IssuanceKeyResult { key_identifier, pop }| {
                // We assume here the WP gave us valid JWTs, and leave it up to the issuer to verify these.
                let header = pop
                    .dangerous_parse_header_unverified()
                    .map_err(WalletIssuanceError::JwtParse)?;

                let public_key = header.public_key().map_err(WalletIssuanceError::JwkConversion)?;

                Ok(((key_identifier, public_key), pop))
            })
            .collect::<Result<VecNonEmpty<_>, WalletIssuanceError>>()?
            .into_nonempty_iter()
            .unzip();

        // Send the proofs of possession to the issuer in a Credential Request to actually fetch the credential copies.
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
                &CredentialRequest::new(offered_credential.to_request_identifier(), proofs),
                &dpop_header,
                &self.session_state.access_token,
            )
            .await?;

        // Extract the credentials from the request and verify each of the copies.
        let credentials = credential_response
            .into_immediate_credentials()
            .ok_or(WalletIssuanceError::DeferredIssuanceUnsupported)?;

        let offered_metadata = self
            .session_state
            .metadata
            .get(&offered_credential.config_id)
            .expect("`IssuanceState::metadata` has an entry for every offered configuration");

        let (credential_copies, first_credential_payload, extended_attestation_types, issued_metadata) =
            match (offered_credential.format, offered_metadata) {
                (Format::SdJwt, metadata @ OfferedCredentialMetadata::TypeMetadata { normalized, raw }) => {
                    let (sd_jwts, credential_payloads) = credentials.into_issued_sd_jwts(
                        key_ids_and_public_keys,
                        metadata,
                        credential_preview,
                        trust_anchors,
                    )?;

                    // Verify that all credentials contain the same metadata integrity value and validate this against
                    // the SD-JWT VC Type Metadata document chain.
                    let verified_metadata = verify_metadata_integrity(sd_jwts.iter(), raw.clone())?;

                    // Credential Metadata is not covered by an integrity digest, nor does it describe a chain of
                    // extended attestation types.
                    (
                        IssuedCredentialCopies::SdJwt(sd_jwts),
                        credential_payloads.into_first(),
                        normalized.extended_vcts().map(String::from).collect(),
                        IssuedCredentialMetadata::TypeMetadata(verified_metadata),
                    )
                }
                (Format::SdJwt, metadata @ OfferedCredentialMetadata::CredentialMetadata(credential_metadata)) => {
                    let (sd_jwts, credential_payloads) = credentials.into_issued_sd_jwts(
                        key_ids_and_public_keys,
                        metadata,
                        credential_preview,
                        trust_anchors,
                    )?;

                    (
                        IssuedCredentialCopies::SdJwt(sd_jwts),
                        credential_payloads.into_first(),
                        Vec::new(),
                        IssuedCredentialMetadata::CredentialMetadata(credential_metadata.clone()),
                    )
                }
                (Format::MsoMdoc, OfferedCredentialMetadata::CredentialMetadata(credential_metadata)) => {
                    let (mdocs, credential_payloads) = credentials.into_issued_mdocs(
                        key_ids_and_public_keys,
                        credential_preview,
                        credential_metadata,
                        trust_anchors,
                    )?;

                    (
                        IssuedCredentialCopies::Mdoc(mdocs),
                        credential_payloads.into_first(),
                        Vec::new(),
                        IssuedCredentialMetadata::CredentialMetadata(credential_metadata.clone()),
                    )
                }
                (Format::MsoMdoc, OfferedCredentialMetadata::TypeMetadata { .. }) => {
                    // The combination of an offered mdoc credential and SD-JWT VC Type Metadata being present in
                    // `IssuanceState` should never occur for the following reasons:
                    //
                    // 1. SD-JWT VC Type Metadata is only fetched for Credential Configurations with the SD-JWT format.
                    // 2. The format of each offered credential is that of its Credential Configuration. When previews
                    //    are received, the format of each preview is matched against the Credential Configuration with
                    //    the same Credential Configuration Identifier. On a mismatch an error is returned.
                    //
                    // This means that the metadata stored for a particular offered credential's Credential
                    // Configuration Identifier should always be appropriate for its format.
                    //
                    // The error variant returned is categorized as `impossible`. Note that this does not include any
                    // details, as these could reveal personal data.
                    return Err(WalletIssuanceError::MdocPreviewWithSdJwtVcTypeMetadata);
                }
            };

        let credential_with_metadata = CredentialWithMetadata::new(
            credential_copies,
            first_credential_payload.previewable_payload.attestation_type,
            first_credential_payload.previewable_payload.expires,
            first_credential_payload.previewable_payload.not_before,
            extended_attestation_types,
            issued_metadata,
            self.session_state.issuer_registration.clone(),
        );

        Ok(credential_with_metadata)
    }
}

impl<H: VcMessageClient> IssuanceSession for HttpIssuanceSession<H> {
    async fn accept_issuance<W>(
        &mut self,
        max_copy_count: NonZeroU8,
        trust_anchors: &TrustAnchors,
        wscd: &W,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>
    where
        W: IssuanceWscd,
    {
        // Request as many copies as the Issuer Metadata will allow, capped by `max_copy_count`.
        let copy_count = std::cmp::min(self.session_state.batch_size, max_copy_count);

        // Determine the proof nonce and DPoP nonce for each credential. If the nonce endpoint is defined in the
        // metadata, call it for each credential in parallel in order to retrieve a proof nonce to be used in the Proof
        // of Possession of the holder key.
        let credential_count = self.session_state.offered_credentials.len().get();
        let (proof_nonces, dpop_nonces): (Vec<_>, Vec<_>) =
            try_join_all((0..credential_count).map(async |_| -> Result<_, WalletIssuanceError> {
                let (proof_nonce, dpop_nonce) = match self.session_state.issuer_endpoints.nonce_endpoint.as_ref() {
                    None => {
                        // There is no nonce endpoint available, so use the DPoP nonce from the Token Response, if
                        // provided.
                        (None, self.session_state.dpop_nonce.clone())
                    }
                    Some(nonce_endpoint) => {
                        let (NonceResponse { c_nonce }, dpop_nonce) = self
                            .message_client
                            .request_nonce(nonce_endpoint.clone().into_url())
                            .await?;

                        // If the nonce endpoint response included a "DPoP-Nonce" header, return that value as the DPoP
                        // nonce. Otherwise, use the value received in the Token Response, if any.
                        let dpop_nonce = dpop_nonce.or_else(|| self.session_state.dpop_nonce.clone());

                        (Some(c_nonce), dpop_nonce)
                    }
                };

                Ok((proof_nonce, dpop_nonce))
            }))
            .await?
            .into_iter()
            .unzip();

        // Have the WSCD generate sets of as many private keys and proofs as the number of credential copies for each
        // individual credential.
        let aud = self.session_state.credential_issuer.as_ref().to_string();
        let key_counts_and_nonces = proof_nonces
            .into_iter()
            .map(|proof_nonce| (copy_count, proof_nonce))
            .collect_vec()
            .try_into()
            .expect("credential_count is non-zero, which guarantees that proof_nonces is non-empty");
        let key_results = wscd
            .perform_issuance(aud, key_counts_and_nonces)
            .await
            .map_err(|e| WalletIssuanceError::PrivateKeyGeneration(e.into()))?;

        // Fetch a set of credential copies for each credential in parallel, using the key identifiers / proofs and a
        // possible DPoP nonce.
        let credentials = try_join_all(
            self.session_state
                .offered_credentials
                .iter()
                .zip_eq(key_results)
                .zip_eq(dpop_nonces)
                .map(|(((offered_credential, preview), keys), dpop_nonce)| {
                    self.fetch_credential(offered_credential, preview, keys, dpop_nonce, trust_anchors)
                }),
        )
        .await?;

        Ok(credentials)
    }

    fn previews_with_metadata(&self) -> Option<impl Iterator<Item = OfferedCredentialPreview<'_>>> {
        match &self.session_state.offered_credentials {
            OfferedCredentials::WithPreviews(creds) => {
                Some(creds.iter().map(|(offered_credential, credential_payload)| {
                    let metadata = self
                        .session_state
                        .metadata
                        .get(&offered_credential.config_id)
                        .expect("`IssuanceState::metadata` has an entry for every offered configuration");

                    OfferedCredentialPreview {
                        format: offered_credential.format,
                        credential_payload,
                        metadata,
                    }
                }))
            }
            OfferedCredentials::WithoutPreviews(_) => None,
        }
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        &self.session_state.issuer_registration
    }
}

impl Credentials {
    /// Create a set of mdoc credentials out of the credential response, along with the payload of each copy. This also
    /// verifies the credentials.
    fn into_issued_mdocs(
        self,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        preview: Option<&PreviewableCredentialPayload>,
        credential_metadata: &CredentialMetadata,
        trust_anchors: &TrustAnchors,
    ) -> Result<(VecNonEmpty<MdocCopy>, VecNonEmpty<CredentialPayload>), WalletIssuanceError> {
        let Self::MsoMdoc(mdoc_credentials) = self else {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: Format::MsoMdoc,
                actual: self.format(),
            });
        };

        let mdocs_and_payloads = mdoc_credentials
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

                // Construct the new mdoc; this also verifies it against the trust anchors.
                let mdoc = Mdoc::new(issuer_signed, &TimeGenerator, trust_anchors)
                    .map_err(WalletIssuanceError::MdocVerification)?;

                let issued_credential_payload = CredentialPayload::from_mdoc(mdoc.clone())?;

                Self::validate_credential(preview, &public_key, &issued_credential_payload, credential_metadata)?;

                Ok((MdocCopy { key_identifier, mdoc }, issued_credential_payload))
            })
            .collect::<Result<VecNonEmpty<_>, _>>()?
            .into_nonempty_iter()
            .unzip();

        Ok(mdocs_and_payloads)
    }

    /// Create a set of SD-JWT credentials out of the credential response, along with the payload of each copy. This
    /// also verifies the credentials.
    fn into_issued_sd_jwts(
        self,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        metadata: &OfferedCredentialMetadata,
        preview: Option<&PreviewableCredentialPayload>,
        trust_anchors: &TrustAnchors,
    ) -> Result<(VecNonEmpty<SdJwtCopy>, VecNonEmpty<CredentialPayload>), WalletIssuanceError> {
        let Self::SdJwt(sd_jwt_credentials) = self else {
            return Err(WalletIssuanceError::UnexpectedCredentialResponseType {
                expected: Format::SdJwt,
                actual: self.format(),
            });
        };

        let sd_jwts_and_payloads = sd_jwt_credentials
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

                if let OfferedCredentialMetadata::TypeMetadata { normalized, .. } = metadata {
                    let issued_claims = issued_credential_payload
                        .previewable_payload
                        .attributes
                        .claim_paths(AttributesTraversalBehaviour::OnlyLeaves);

                    // Verify whether each claims selective disclosability matches the metadata.
                    Self::verify_selective_disclosability(&sd_jwt, issued_claims, normalized)?;
                }

                Self::validate_credential(preview, &public_key, &issued_credential_payload, metadata)?;

                Ok((SdJwtCopy { key_identifier, sd_jwt }, issued_credential_payload))
            })
            .collect::<Result<VecNonEmpty<_>, WalletIssuanceError>>()?
            .into_nonempty_iter()
            .unzip();

        Ok(sd_jwts_and_payloads)
    }

    fn validate_credential(
        preview: Option<&PreviewableCredentialPayload>,
        holder_pubkey: &PublicKey,
        credential_payload: &CredentialPayload,
        metadata: &impl AttestationClaims,
    ) -> Result<(), WalletIssuanceError> {
        if credential_payload.confirmation_key.try_to_public_key()? != *holder_pubkey {
            return Err(WalletIssuanceError::PublicKeyMismatch);
        }

        if let Some(preview) = preview {
            // Check that the credential contains exactly the attributes the issuer said it would have.
            if credential_payload.previewable_payload != *preview {
                return Err(WalletIssuanceError::IssuedCredentialMismatch {
                    actual: Box::new(credential_payload.previewable_payload.clone()),
                    expected: Box::new(preview.clone()),
                });
            }
        }

        // Check that those attributes are the ones described by the metadata of the credential configuration. Note
        // that this covers the attributes of the preview as well, as the two are equal at this point.
        credential_payload
            .previewable_payload
            .attributes
            .validate(metadata)
            .map_err(WalletIssuanceError::AttributesVerification)?;

        Ok(())
    }

    fn verify_selective_disclosability(
        sd_jwt: &VerifiedSdJwt,
        issued_claims: Vec<VecNonEmpty<ClaimPath>>,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<(), WalletIssuanceError> {
        // Note that this reads the selective disclosability of each claim, which is specific to SD-JWT VC Type
        // Metadata and therefore not available through the `AttestationMetadata` trait.
        let sd_metadata = metadata
            .claims()
            .iter()
            .map(|claim| (claim.path.as_ref().to_vec(), claim.sd))
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

/// Verify that each credential copy contains the same metadata integrity value, use this to validate a SD-JWT VC
/// Type Metadata document chain and return the resulting `VerifiedTypeMetadataDocuments` type.
fn verify_metadata_integrity<'a>(
    sd_jwts: impl Iterator<Item = &'a SdJwtCopy>,
    metadata_documents: SortedTypeMetadataDocuments,
) -> Result<VerifiedTypeMetadataDocuments, WalletIssuanceError> {
    // Verify that each of the resulting credentials contain exactly the same metadata integrity digest.
    let unique_integrities: HashSet<_> = sd_jwts
        .map(|sd_jwt_copy| {
            sd_jwt_copy
                .sd_jwt
                .claims()
                .vct_integrity
                .as_ref()
                .ok_or(WalletIssuanceError::MetadataIntegrityMissing)
        })
        .try_collect()?;

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

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::num::NonZeroU8;
    use std::time::Duration;
    use std::vec;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::Attributes;
    use attestation_data::credential_payload::PreviewableCredentialPayload;
    use attestation_types::credential_format::Format;
    use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::status_claim::StatusListClaim;
    use chrono::Utc;
    use cose::TypedCose;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use derive_more::Debug;
    use futures::FutureExt;
    use jwt::jwk::jwk_to_public_key;
    use jwt::nonce::Nonce;
    use mdoc::IdentifierListInfo;
    use mdoc::IssuerSigned;
    use mdoc::MdocStatus;
    use mdoc::utils::serialization::TaggedBytes;
    use mockall::predicate::eq;
    use oauth::errors::ErrorResponse;
    use oauth::errors::RemoteErrorCode;
    use oauth::issuer_identifier::IssuerIdentifier;
    use oauth::metadata::well_known::WellKnownMetadata;
    use oauth::token::TokenType;
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
    use wscd::wia::mock::MockWiaClient;

    use super::*;
    use crate::authorization_details::AuthorizationDetails;
    use crate::credential::CredentialRequestProofs;
    use crate::metadata::issuer_metadata::CredentialFormat;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::metadata::oauth_metadata::IssuerAuthorizationServerMetadata;
    use crate::preview::CredentialPreviewResponse;
    use crate::token::CredentialPreview;
    use crate::token::VciTokenRequest;
    use crate::token::VciTokenResponse;
    use crate::wallet_issuance::TypeMetadataChainError;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::mock::RecordingWiaClient;

    impl<H> HttpIssuanceSession<H> {
        pub fn batch_size(&self) -> NonZeroU8 {
            self.session_state.batch_size
        }
    }

    fn invalid_grant_error() -> WalletIssuanceError {
        WalletIssuanceError::VciTokenRequest(Box::new(ErrorResponse {
            error: RemoteErrorCode::Known(VciTokenErrorCode::InvalidGrant),
            error_description: None,
            error_uri: None,
        }))
    }

    #[test]
    fn map_pre_authorized_token_error_translates_only_pre_authorized_invalid_grant() {
        use oauth::token::AuthorizationCode;
        let pre_authorized = VciTokenRequest::new_pre_authorized(AuthorizationCode::from("the-code".to_string()));
        let authorization_code = VciTokenRequest::new_authorization_code(
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
            WalletIssuanceError::VciTokenRequest(_)
        );

        // Any other error code in the pre-authorized flow is left untouched.
        let other = WalletIssuanceError::VciTokenRequest(Box::new(ErrorResponse {
            error: RemoteErrorCode::Known(VciTokenErrorCode::InvalidRequest),
            error_description: None,
            error_uri: None,
        }));
        assert_matches!(
            map_pre_authorized_token_error(other, &pre_authorized),
            WalletIssuanceError::VciTokenRequest(_)
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
        let oauth_metadata = IssuerAuthorizationServerMetadata::new_mock(issuer_metadata.issuer_identifier().clone());
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

        let error = HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            oauth_metadata.oauth_metadata.token_endpoint,
            mechanism,
            VciTokenRequest::new_mock(),
            &wia_client,
            &oauth_metadata.oauth_metadata.issuer,
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
        issuer_metadata: IssuerMetadata,
        preview_payloads: Vec<(
            CredentialId,
            CredentialConfigurationId,
            Format,
            PreviewableCredentialPayload,
        )>,
        type_metadata: TypeMetadata,
        token_response_fields: &TokenResponseFields,
    ) -> Result<HttpIssuanceSession<MockVcMessageClient>, WalletIssuanceError> {
        let authorization_details = match &token_response_fields {
            TokenResponseFields::AuthorizationDetails(identifiers) | TokenResponseFields::Both(identifiers, _) => {
                let (config_ids, credential_ids): (Vec<_>, Vec<_>) = identifiers
                    .iter()
                    .flat_map(|(config_id, credential_ids)| {
                        credential_ids.iter().map(|credential_id| {
                            (
                                CredentialConfigurationId::from(config_id.to_string()),
                                credential_id.to_string().into(),
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
                let token_response = VciTokenResponse {
                    oauth_response: oauth::token::TokenResponse {
                        access_token: "access_token".to_string().into(),
                        token_type: TokenType::DPoP,
                        expires_in: None,
                        refresh_token: None,
                        scope,
                    },
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

        // The Credential Preview endpoint should never be called if the issuer does not have one.
        let preview_call_count = if issuer_metadata.endpoints.credential_preview_endpoint.is_some() {
            0..=1
        } else {
            0..=0
        };
        mock_msg_client
            .expect_request_credential_preview()
            .times(preview_call_count)
            .return_once(move |_url, _access_token| {
                let previews = preview_payloads
                    .into_iter()
                    .map(
                        |(credential_id, config_id, format, preview_payload)| CredentialPreview {
                            credential_id,
                            config_id,
                            format,
                            credential_payload: preview_payload,
                        },
                    )
                    .collect_vec()
                    .try_into()
                    .unwrap();

                Ok(CredentialPreviewResponse {
                    credential_previews: previews,
                })
            });

        let oauth_metadata = IssuerAuthorizationServerMetadata::new_mock(issuer_metadata.issuer_identifier().clone());

        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        HttpIssuanceSession::create(
            mock_msg_client,
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            oauth_metadata.oauth_metadata.token_endpoint,
            ClientAttestationChallengeMechanism::ChallengeEndpoint(
                oauth_metadata
                    .client_attestation_metadata_extension
                    .challenge_endpoint
                    .unwrap(),
            ),
            VciTokenRequest::new_mock(),
            &MockWiaClient::new(),
            &oauth_metadata.oauth_metadata.issuer,
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
            IssuerMetadata::new_mock("https://example.com".parse().unwrap(), credential_configs),
            vec![(
                "credential_id".to_string().into(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect("starting issuance session should succeed");

        let Ok(preview) = session.previews_with_metadata().unwrap().exactly_one() else {
            panic!("issuance session should contain exactly one preview")
        };

        assert_matches!(
                &preview.credential_payload.attributes.as_ref()["family_name"],
                Attribute::Text(v) if v == "De Bruijn");

        let OfferedCredentialMetadata::TypeMetadata { normalized, .. } = preview.metadata else {
            panic!("session should contain type metadata for the credential configuration");
        };

        assert_eq!(
            *normalized,
            TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example())
                .2
                .into_normalized(&preview.credential_payload.attestation_type)
                .unwrap()
                .0
        );
    }

    #[rstest]
    #[case::authorization_details(
        TokenResponseFields::AuthorizationDetails(vec![("config_id", vec!["credential_id_1", "credential_id_2"])]),
        vec![
            OfferedCredential::new_by_credential_id(
                "credential_id_1".to_string().into(),
                "config_id".to_string().into(),
                Format::SdJwt,
            ),
            OfferedCredential::new_by_credential_id(
                "credential_id_2".to_string().into(),
                "config_id".to_string().into(),
                Format::SdJwt,
            ),
        ]
    )]
    #[case::scope(
        TokenResponseFields::Scope(vec!["config_id_scope"]),
        vec![OfferedCredential::new_by_config_id("config_id".to_string().into(), Format::SdJwt)]
    )]
    #[case::no_authorization_details_or_scope(
        TokenResponseFields::Neither,
        vec![OfferedCredential::new_by_config_id("config_id".to_string().into(), Format::SdJwt)]
    )]
    fn test_start_issuance_without_preview_endpoint(
        #[case] token_response_fields: TokenResponseFields,
        #[case] expected_offered_credentials: Vec<OfferedCredential>,
    ) {
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let mut issuer_metadata = IssuerMetadata::new_mock(
            "https://example.com".parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        issuer_metadata.endpoints.credential_preview_endpoint = None;

        let session = test_start_issuance(
            issuer_metadata,
            vec![],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect("starting issuance session should succeed");

        assert!(session.previews_with_metadata().is_none());

        // Without previews, the offered credentials are determined by the Credential Configurations and the Token
        // Response only.
        let OfferedCredentials::WithoutPreviews(offered_credentials) = &session.session_state.offered_credentials
        else {
            panic!("issuance session should not contain previews");
        };
        assert_eq!(
            offered_credentials.iter().collect::<HashSet<_>>(),
            expected_offered_credentials.iter().collect::<HashSet<_>>()
        );

        // The metadata of the Credential Configuration should still be fetched.
        assert_matches!(
            session.session_state.metadata.get(&config_id),
            Some(OfferedCredentialMetadata::TypeMetadata { .. })
        );
    }

    #[test]
    fn test_start_issuance_authorization_details_duplicate_credential_ids() {
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![
                    (
                        CredentialConfigurationId::from("mdoc_config_id".to_string()),
                        CredentialKind::new(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string()),
                    ),
                    (
                        CredentialConfigurationId::from("sd_jwt_config_id".to_string()),
                        CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                    ),
                ],
            ),
            vec![
                (
                    "credential_id".to_string().into(),
                    CredentialConfigurationId::from("mdoc_config_id".to_string()),
                    Format::MsoMdoc,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id".to_string().into(),
                    CredentialConfigurationId::from("sd_jwt_config_id".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
            ],
            TypeMetadata::pid_example(),
            &TokenResponseFields::AuthorizationDetails(vec![
                ("mdoc_config_id", vec!["credential_id"]),
                ("sd_jwt_config_id", vec!["credential_id"]),
            ]),
        )
        .expect_err("starting issuance session should fail");

        let expected_duplicates = HashMap::from([(
            "credential_id".to_string().into(),
            HashSet::from([
                CredentialConfigurationId::from("mdoc_config_id".to_string()),
                CredentialConfigurationId::from("sd_jwt_config_id".to_string()),
            ]),
        )]);
        assert_matches!(
            error,
            WalletIssuanceError::AuthorizationDetailsDuplicateCredentialIds(duplicates)
                if duplicates == expected_duplicates
        );
    }

    #[test]
    fn test_start_issuance_authorization_details_unknown_credential_config_ids() {
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string().into(),
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
            WalletIssuanceError::AuthorizationDetailsUnknownCredentialConfigIds(config_ids)
                if config_ids == vec![CredentialConfigurationId::from("unknown_config_id".to_string())]
        );
    }

    #[test]
    fn test_start_issuance_token_response_empty_scope() {
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string().into(),
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
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string().into(),
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
    fn test_start_issuance_type_metadata_verification_error() {
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    config_id.clone(),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![(
                "credential_id".to_string().into(),
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
    fn test_start_issuance_metadata_missing() {
        // Create issuer metadata for an SD-JWT configuration that has neither a type metadata URI nor Credential
        // Metadata to fall back on.
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
            .for_each(|config| {
                config.type_metadata_uri = None;
                config.credential_metadata = None;
            });

        let error = test_start_issuance(
            issuer_metadata,
            vec![(
                "credential_id".to_string().into(),
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
            WalletIssuanceError::MetadataMissing(missing_config_ids) if missing_config_ids == vec![config_id]
        );
    }

    /// Create mock issuer metadata where the configurations with the specified IDs have an unsupported format.
    fn issuer_metadata_with_unsupported_formats(
        config_ids: Vec<CredentialConfigurationId>,
        unsupported_config_ids: &[CredentialConfigurationId],
    ) -> IssuerMetadata {
        let mut issuer_metadata = IssuerMetadata::new_mock(
            "https://example.com".parse().unwrap(),
            config_ids
                .into_iter()
                .map(|config_id| {
                    (
                        config_id,
                        CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                    )
                })
                .collect(),
        );

        for config_id in unsupported_config_ids {
            issuer_metadata
                .credential_configurations_supported
                .get_mut(config_id)
                .unwrap()
                .format = CredentialFormat::Other {
                format: "jwt_vc_json".to_string(),
            };
        }

        issuer_metadata
    }

    #[rstest]
    #[case::authorization_details(TokenResponseFields::AuthorizationDetails(vec![
        ("config_id", vec!["credential_id"]),
        ("unsupported_config_id", vec!["unsupported_credential_id"]),
    ]))]
    #[case::scope(TokenResponseFields::Scope(vec!["config_id_scope", "unsupported_config_id_scope"]))]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither)]
    fn test_start_issuance_unsupported_format_discarded(#[case] token_response_fields: TokenResponseFields) {
        let config_id = CredentialConfigurationId::from("config_id".to_string());
        let unsupported_config_id = CredentialConfigurationId::from("unsupported_config_id".to_string());
        let issuer_metadata = issuer_metadata_with_unsupported_formats(
            vec![config_id.clone(), unsupported_config_id.clone()],
            &[unsupported_config_id],
        );

        // The issuer only provides a preview for the configuration with a supported format.
        let session = test_start_issuance(
            issuer_metadata,
            vec![(
                "credential_id".to_string().into(),
                config_id.clone(),
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect("starting issuance session should succeed");

        let Ok((offered_credential, _preview)) = session.session_state.offered_credentials.iter().exactly_one() else {
            panic!("issuance session should contain exactly one offered credential")
        };
        assert_eq!(offered_credential.config_id, config_id);
    }

    #[rstest]
    #[case::authorization_details(
        TokenResponseFields::AuthorizationDetails(vec![("unsupported_config_id", vec!["unsupported_credential_id"])])
    )]
    #[case::scope(TokenResponseFields::Scope(vec!["unsupported_config_id_scope"]))]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither)]
    fn test_start_issuance_error_no_supported_format(#[case] token_response_fields: TokenResponseFields) {
        let unsupported_config_id = CredentialConfigurationId::from("unsupported_config_id".to_string());
        let issuer_metadata = issuer_metadata_with_unsupported_formats(
            vec![unsupported_config_id.clone()],
            std::slice::from_ref(&unsupported_config_id),
        );

        let error = test_start_issuance(
            issuer_metadata,
            vec![(
                "unsupported_credential_id".to_string().into(),
                unsupported_config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect_err("starting issuance session should not succeed");

        assert_matches!(error, WalletIssuanceError::NoSupportedCredentialConfigurations);
    }

    #[test]
    fn test_start_issuance_sd_jwt_credential_metadata_fallback() {
        // Create issuer metadata for an SD-JWT configuration that has no type metadata URI, but does have
        // Credential Metadata to fall back on.
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
            .for_each(|config| {
                config.type_metadata_uri = None;
                config.credential_metadata = Some(CredentialMetadata::new_example(&["family_name"]));
            });

        let session = test_start_issuance(
            issuer_metadata,
            vec![(
                "credential_id".to_string().into(),
                config_id,
                Format::SdJwt,
                PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
            )],
            TypeMetadata::pid_example(),
            &TokenResponseFields::Neither,
        )
        .expect("starting issuance session should succeed");

        let Ok(preview) = session.previews_with_metadata().unwrap().exactly_one() else {
            panic!("issuance session should contain exactly one preview")
        };

        // The SD-JWT configuration has no type metadata URI, so it should fall back to its Credential Metadata.
        assert_matches!(preview.metadata, OfferedCredentialMetadata::CredentialMetadata(_));
    }

    #[test]
    fn test_start_issuance_type_metadata_host_mismatch() {
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
            issuer_metadata,
            vec![(
                "credential_id".to_string().into(),
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
    fn test_fetch_metadata_shared_type_metadata_uri() {
        let issuer_identifier: IssuerIdentifier = "https://example.com".parse().unwrap();
        let pid_config_id = CredentialConfigurationId::from("pid_config_id".to_string());
        let other_config_id = CredentialConfigurationId::from("other_config_id".to_string());

        // Both configurations carry the same type metadata URI and have the same vct value.
        let issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier.clone(),
            vec![(
                pid_config_id.clone(),
                CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
            )],
        );
        let config = SupportedConfiguration::try_new(
            issuer_metadata
                .credential_configurations_supported
                .get(&pid_config_id)
                .unwrap()
                .clone(),
        )
        .unwrap();

        // The metadata should be fetched only once.
        let mut mock_msg_client = MockVcMessageClient::new();
        mock_msg_client
            .expect_request_type_metadata()
            .times(1)
            .returning(|_url| {
                let (_, _, documents) = TypeMetadataDocuments::from_single_example(TypeMetadata::pid_example());
                Ok(documents)
            });

        let metadata = HttpIssuanceSession::fetch_metadata(
            vec![(&pid_config_id, &config), (&other_config_id, &config)],
            &issuer_identifier,
            &mock_msg_client,
        )
        .now_or_never()
        .unwrap()
        .expect("fetching metadata should succeed");

        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains_key(&pid_config_id));
        assert!(metadata.contains_key(&other_config_id));

        // The normalized SD-JWT VC Type Metadata for both Credential Configurations should be exactly the same.
        match (metadata.get(&pid_config_id), metadata.get(&other_config_id)) {
            (
                Some(OfferedCredentialMetadata::TypeMetadata {
                    normalized: pid_normalized,
                    ..
                }),
                Some(OfferedCredentialMetadata::TypeMetadata {
                    normalized: other_normalized,
                    ..
                }),
            ) => {
                assert_eq!(pid_normalized, other_normalized);
            }
            _ => {
                panic!("metadata for both credential configurations should be SD-JWT VC Type Metadata");
            }
        }
    }

    #[test]
    fn test_start_issuance_type_metadata_multiple_vcts() {
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
            issuer_metadata,
            vec![
                (
                    "pid_credential_id".to_string().into(),
                    pid_config_id,
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "address_credential_id".to_string().into(),
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
            WalletIssuanceError::TypeMetadataUriMultipleVcts(multi_vct_uris)
                if multi_vct_uris.len() == 1 &&
                    multi_vct_uris.first().unwrap().0 == expected_type_metadata_uri &&
                    multi_vct_uris
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
            ("config_id_2", vec!["credential_id_2"]),
            ("config_id_3", vec!["credential_id_3"])
        ])
    )]
    #[case::scope(TokenResponseFields::Scope(vec!["config_id_1_scope", "config_id_2_scope", "config_id_3_scope"]))]
    #[case::no_authorization_details_or_scope(TokenResponseFields::Neither)]
    fn test_start_issuance_error_preview_missing_credential(#[case] token_response_fields: TokenResponseFields) {
        let error = test_start_issuance(
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
                    (
                        CredentialConfigurationId::from("config_id_3".to_string()),
                        CredentialKind::new(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string()),
                    ),
                ],
            ),
            vec![
                (
                    "credential_id_1".to_string().into(),
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_3".to_string().into(),
                    CredentialConfigurationId::from("config_id_3".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
            ],
            TypeMetadata::pid_example(),
            &token_response_fields,
        )
        .expect_err("starting issuance session should fail");

        let (expected_credential_id_2, expected_credential_id_3) = match token_response_fields {
            TokenResponseFields::AuthorizationDetails(_) | TokenResponseFields::Both(_, _) => (
                Some("credential_id_2".to_string().into()),
                Some("credential_id_3".to_string().into()),
            ),
            TokenResponseFields::Scope(_) | TokenResponseFields::Neither => (None, None),
        };
        let expected_missing = HashSet::from([
            (
                "config_id_2".to_string().into(),
                expected_credential_id_2,
                Format::SdJwt,
            ),
            (
                "config_id_3".to_string().into(),
                expected_credential_id_3,
                Format::MsoMdoc,
            ),
        ]);
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
        let error = test_start_issuance(
            IssuerMetadata::new_mock(
                "https://example.com".parse().unwrap(),
                vec![(
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    CredentialKind::new(Format::SdJwt, PID_ATTESTATION_TYPE.to_string()),
                )],
            ),
            vec![
                (
                    "credential_id_1_1".to_string().into(),
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    Format::MsoMdoc,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_1_1".to_string().into(),
                    CredentialConfigurationId::from("config_id_1".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_2_1".to_string().into(),
                    CredentialConfigurationId::from("config_id_2".to_string()),
                    Format::SdJwt,
                    PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
                ),
                (
                    "credential_id_1_2".to_string().into(),
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
            (
                "config_id_1".to_string().into(),
                "credential_id_1_1".to_string().into(),
                Format::MsoMdoc,
            ),
            (
                "config_id_2".to_string().into(),
                "credential_id_2_1".to_string().into(),
                Format::SdJwt,
            ),
            (
                "config_id_1".to_string().into(),
                "credential_id_1_2".to_string().into(),
                Format::SdJwt,
            ),
        ];
        assert_matches!(error, WalletIssuanceError::PreviewExcessCredentials(excess) if excess == expected_excess);
    }

    /// Return a new session ready for `accept_issuance()`. If `has_previews` is `false`, the previews are only used to
    /// determine the offered credentials and then discarded, as if the issuer had no Credential Preview endpoint.
    fn new_session_state(
        credential_previews: Vec<CredentialPreview>,
        metadata: HashMap<CredentialConfigurationId, OfferedCredentialMetadata>,
        batch_size: NonZeroU8,
        has_nonce_endpoint: bool,
        has_previews: bool,
    ) -> IssuanceState {
        let issuer_identifier = "https://issuer.example.com".parse().unwrap();

        let mut issuer_endpoints = IssuerEndpoints::new_mock(&issuer_identifier);
        if !has_nonce_endpoint {
            issuer_endpoints.nonce_endpoint = None;
        }
        if !has_previews {
            issuer_endpoints.credential_preview_endpoint = None;
        }

        let offered_credentials_and_previews = credential_previews.into_iter().map(|preview| {
            let offered_credential = OfferedCredential::new_by_credential_id(
                preview.credential_id.clone(),
                preview.config_id.clone(),
                preview.format,
            );

            (offered_credential, preview.credential_payload)
        });

        let offered_credentials = if has_previews {
            OfferedCredentials::WithPreviews(offered_credentials_and_previews.collect_vec().try_into().unwrap())
        } else {
            OfferedCredentials::WithoutPreviews(
                NESet::try_from_set(
                    offered_credentials_and_previews
                        .map(|(offered_credential, _preview)| offered_credential)
                        .collect(),
                )
                .unwrap(),
            )
        };

        IssuanceState {
            access_token: "access_token".to_string().into(),
            credential_issuer: issuer_identifier,
            issuer_endpoints,
            batch_size,
            offered_credentials,
            metadata,
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
        formats_by_credential_id: HashMap<CredentialId, Format>,
        pub trust_anchors: TrustAnchors,
        issuer_key: KeyPair,
        metadata_integrity: Integrity,
        /// mdoc attributes, namespaced (namespace, element).
        mdoc_previewable_payload: PreviewableCredentialPayload,
        /// The same attributes as `previewable_payload` (without mdoc namesapce) and matching
        /// `NormalizedTypeMetadata`.
        sd_jwt_previewable_payload: PreviewableCredentialPayload,
        normalized_metadata: NormalizedTypeMetadata,
        pub first_metadata_integrity_random: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum SdJwtMetadataUsage {
        CredentialMetadata,
        TypeMetadata,
    }

    impl MockCredentialSigner {
        /// A credential configuration holds a single format, so each format offered gets its own configuration.
        fn config_id_for_format(format: Format) -> CredentialConfigurationId {
            format.to_string().into()
        }

        pub fn new_with_preview_and_type_metadata(
            formats_by_credential_id: HashMap<CredentialId, Format>,
            sd_jwt_uses_credential_metadata: SdJwtMetadataUsage,
        ) -> (
            Self,
            Vec<CredentialPreview>,
            HashMap<CredentialConfigurationId, OfferedCredentialMetadata>,
        ) {
            let sd_jwt_preview_payload =
                PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());
            let mdoc_preview_payload =
                PreviewableCredentialPayload::example_family_name_mdoc(&MockTimeGenerator::default());
            let type_metadata =
                TypeMetadata::example_with_claim_name(&sd_jwt_preview_payload.attestation_type, "family_name");

            Self::from_metadata_and_preview(
                formats_by_credential_id,
                type_metadata,
                mdoc_preview_payload,
                sd_jwt_preview_payload,
                sd_jwt_uses_credential_metadata,
            )
        }

        fn from_metadata_and_preview(
            formats_by_credential_id: HashMap<CredentialId, Format>,
            type_metadata: TypeMetadata,
            mdoc_previewable_payload: PreviewableCredentialPayload,
            sd_jwt_previewable_payload: PreviewableCredentialPayload,
            sd_jwt_metadata_usage: SdJwtMetadataUsage,
        ) -> (
            Self,
            Vec<CredentialPreview>,
            HashMap<CredentialConfigurationId, OfferedCredentialMetadata>,
        ) {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            let trust_anchors = TrustAnchors::try_from(vec![ca.to_borrowing_trust_anchor()]).unwrap();

            let issuer_key = ca.generate_pid_issuer_mock().unwrap();

            let (attestation_type, metadata_integrity, metadata_documents) =
                TypeMetadataDocuments::from_single_example(type_metadata);
            let (normalized_metadata, raw_metadata) = metadata_documents.into_normalized(&attestation_type).unwrap();

            let previews = formats_by_credential_id
                .iter()
                .map(|(credential_id, format)| CredentialPreview {
                    credential_id: credential_id.clone(),
                    config_id: Self::config_id_for_format(*format),
                    format: *format,
                    credential_payload: match format {
                        Format::MsoMdoc => mdoc_previewable_payload.clone(),
                        Format::SdJwt => sd_jwt_previewable_payload.clone(),
                    },
                })
                .collect_vec();

            // Each format is described by the metadata that belongs to it.
            let metadata = formats_by_credential_id
                .values()
                .unique()
                .map(|format| {
                    let metadata = match (format, sd_jwt_metadata_usage) {
                        (Format::MsoMdoc, _) => OfferedCredentialMetadata::CredentialMetadata(
                            CredentialMetadata::new_mdoc_example(&attestation_type, &["family_name"]),
                        ),
                        (Format::SdJwt, SdJwtMetadataUsage::CredentialMetadata) => {
                            OfferedCredentialMetadata::CredentialMetadata(CredentialMetadata::new_example(&[
                                "family_name",
                            ]))
                        }
                        (Format::SdJwt, SdJwtMetadataUsage::TypeMetadata) => OfferedCredentialMetadata::TypeMetadata {
                            normalized: normalized_metadata.clone(),
                            raw: raw_metadata.clone(),
                        },
                    };

                    (Self::config_id_for_format(*format), metadata)
                })
                .collect();

            let signer = Self {
                formats_by_credential_id,
                trust_anchors,
                issuer_key,
                metadata_integrity,
                first_metadata_integrity_random: false,
                mdoc_previewable_payload,
                sd_jwt_previewable_payload,
                normalized_metadata,
            };

            (signer, previews, metadata)
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
            credential_id: &CredentialId,
            holder_pubkeys: impl IntoNonEmptyIterator<Item = &'a PublicKey>,
        ) -> CredentialResponse {
            let format = *self
                .formats_by_credential_id
                .get(credential_id)
                .expect("requested credential identifier is not correct");

            // An mdoc lays its attributes out in a name space, while an SD-JWT carries them as its metadata
            // prescribes, so each format signs its own previewable payload.
            let previewable_payload = match format {
                Format::MsoMdoc => &self.mdoc_previewable_payload,
                Format::SdJwt => &self.sd_jwt_previewable_payload,
            };

            // Type Metadata describes an SD-JWT, so only that format binds to an integrity digest.
            let credential_payloads = holder_pubkeys
                .into_nonempty_iter()
                .enumerate()
                .map(|(index, holder_pubkey)| {
                    let metadata_integrity = matches!(format, Format::SdJwt).then(|| {
                        if self.first_metadata_integrity_random && index == 0 {
                            Integrity::from(crypto::utils::random_bytes(32))
                        } else {
                            self.metadata_integrity.clone()
                        }
                    });

                    CredentialPayload::from_previewable_credential_payload_unvalidated(
                        previewable_payload.clone(),
                        Utc::now(),
                        holder_pubkey,
                        metadata_integrity,
                        Some(StatusListClaim::new_mock()),
                    )
                    .unwrap()
                });

            let credentials = match format {
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
    #[case::credential_id_single_mdoc(AcceptIssuanceTestFormats::CredentialId(vec![Format::MsoMdoc]))]
    #[case::config_id_single_mdoc(AcceptIssuanceTestFormats::CredentialConfigurationId(Format::MsoMdoc))]
    #[case::multi_mdoc(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::MsoMdoc, Format::MsoMdoc, Format::MsoMdoc])
    )]
    #[case::credential_id_single_sd_jwt(AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt]))]
    #[case::config_id_single_sd_jwt(AcceptIssuanceTestFormats::CredentialConfigurationId(Format::SdJwt))]
    #[case::multi_sd_jwt(AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt, Format::SdJwt, Format::SdJwt]))]
    #[case::mixed(
        AcceptIssuanceTestFormats::CredentialId(vec![Format::SdJwt, Format::MsoMdoc, Format::MsoMdoc, Format::SdJwt]),
    )]
    fn test_accept_issuance(
        #[case] formats: AcceptIssuanceTestFormats,
        #[values(NonZeroU8::MIN, 4.try_into().unwrap())] batch_size: NonZeroU8,
        #[values(NonZeroU8::MIN, 5.try_into().unwrap())] max_copy_count: NonZeroU8,
        #[values(
            TestNonceEndpoint::Absent,
            TestNonceEndpoint::Present,
            TestNonceEndpoint::PresentWithDpopNonce
        )]
        nonce_endpoint: TestNonceEndpoint,
        #[values(true, false)] has_previews: bool,
    ) {
        let formats_by_credential_id = match formats {
            AcceptIssuanceTestFormats::CredentialId(formats) => formats
                .into_iter()
                .enumerate()
                .map(|(index, format)| (format!("credential_id_{index}").into(), format))
                .collect(),
            AcceptIssuanceTestFormats::CredentialConfigurationId(format) => {
                HashMap::from([("credential_id".to_string().into(), format)])
            }
        };
        let credential_count = formats_by_credential_id.len();

        let (signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            formats_by_credential_id,
            SdJwtMetadataUsage::TypeMetadata,
        );
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

        let session_state = new_session_state(previews, metadata, batch_size, has_nonce_endpoint, has_previews);
        let expected_issuer_registration = serde_json::to_value(&session_state.issuer_registration).unwrap();

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

        let mut session = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state,
        };

        assert_eq!(session.previews_with_metadata().is_some(), has_previews);

        let credentials = session
            .accept_issuance(max_copy_count, &trust_anchors, &wscd)
            .now_or_never()
            .unwrap()
            .expect("accepting issuance should succeed");

        assert_eq!(credentials.len(), credential_count);

        let expected_copy_count = std::cmp::min(batch_size, max_copy_count).into();
        for credential in &credentials {
            assert_eq!(
                serde_json::to_value(&credential.issuer_registration).unwrap(),
                expected_issuer_registration
            );
            let copy_count = match &credential.copies {
                IssuedCredentialCopies::Mdoc(mdoc_copies) => mdoc_copies.len(),
                IssuedCredentialCopies::SdJwt(sd_jwt_copies) => sd_jwt_copies.len(),
            };

            assert_eq!(copy_count, expected_copy_count);
        }
    }

    #[test]
    fn test_accept_issuance_with_identifier_list_status() {
        let (signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_owned().into(), Format::MsoMdoc)]),
            SdJwtMetadataUsage::CredentialMetadata,
        );
        let trust_anchors = signer.trust_anchors.clone();

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().times(1).return_once(
            move |_url, credential_request, _dpop_header, _access_token_header| {
                // Build a normal response first, to reuse its holder-key and attribute handling, then replace its
                // status with an identifier list and re-sign it.
                let response = signer.response_from_request(credential_request);
                let Credentials::MsoMdoc(mdoc_credentials) = response.into_immediate_credentials().unwrap() else {
                    panic!("response should contain mdoc credentials");
                };
                let issuer_signed = mdoc_credentials.into_first().credential;

                let TaggedBytes(mut mso) = issuer_signed.issuer_auth.dangerous_parse_unverified().unwrap();
                mso.status = Some(MdocStatus::IdentifierList(IdentifierListInfo {
                    id: hex::decode("cccc").unwrap(),
                    uri: "https://example.com/identifierlists/1".parse().unwrap(),
                    certificate: None,
                }));
                let mso_tagged = TaggedBytes(mso);
                let issuer_auth = TypedCose::sign_with_certificate(&mso_tagged, &signer.issuer_key, true)
                    .now_or_never()
                    .unwrap()
                    .unwrap();

                let issuer_signed = IssuerSigned {
                    name_spaces: issuer_signed.name_spaces,
                    issuer_auth,
                };
                let credentials = Credentials::MsoMdoc(vec_nonempty![MdocCredential::new(issuer_signed)]);

                Ok(CredentialResponse::new_immediate(credentials))
            },
        );

        let credentials = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, metadata, NonZeroU8::MIN, true, true),
        }
        .accept_issuance(NonZeroU8::MIN, &trust_anchors, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect("accepting issuance of an mdoc with an identifier list status should succeed");

        let credential = credentials.into_iter().exactly_one().unwrap();
        let IssuedCredentialCopies::Mdoc(mdoc_copies) = credential.copies else {
            panic!("issued credential should be an mdoc");
        };
        assert_eq!(mdoc_copies.len().get(), 1);
    }

    #[test]
    fn test_accept_issuance_sd_jwt_credential_metadata_fallback() {
        let (signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string().into(), Format::SdJwt)]),
            SdJwtMetadataUsage::CredentialMetadata,
        );
        let trust_anchors = signer.trust_anchors.clone();

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().times(1).return_once(
            move |_url, credential_request, _dpop_header, _access_token_header| {
                Ok(signer.response_from_request(credential_request))
            },
        );

        let credential_copies = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, metadata, NonZeroU8::MIN, true, true),
        }
        .accept_issuance(NonZeroU8::MAX, &trust_anchors, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect("accepting issuance should succeed");

        let credential_with_metadata = credential_copies.into_iter().exactly_one().unwrap();

        assert_matches!(credential_with_metadata.copies, IssuedCredentialCopies::SdJwt(_));
        assert_matches!(
            credential_with_metadata.metadata,
            IssuedCredentialMetadata::CredentialMetadata(_)
        );
        // Credential Metadata does not describe a chain of extended attestation types.
        assert!(credential_with_metadata.extended_attestation_types.is_empty());
    }

    #[test]
    fn test_accept_issuance_error_mdoc_described_by_type_metadata() {
        let (signer, previews, mut metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string().into(), Format::MsoMdoc)]),
            SdJwtMetadataUsage::TypeMetadata,
        );
        let trust_anchors = signer.trust_anchors.clone();

        // Replace the mdoc configuration's Credential Metadata with Type Metadata for the same attestation type.
        let config_id = MockCredentialSigner::config_id_for_format(Format::MsoMdoc);
        let (_, _, metadata_documents) = TypeMetadataDocuments::from_single_example(
            TypeMetadata::example_with_claim_name(PID_ATTESTATION_TYPE, "family_name"),
        );
        let (normalized, raw) = metadata_documents.into_normalized(PID_ATTESTATION_TYPE).unwrap();
        metadata.insert(config_id, OfferedCredentialMetadata::TypeMetadata { normalized, raw });

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        // The credential response is only rejected once it has been received, so it is still requested.
        mock_msg_client.expect_request_credential().times(1).return_once(
            move |_url, credential_request, _dpop_header, _access_token| {
                Ok(signer.response_from_request(credential_request))
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, metadata, NonZeroU8::MIN, true, true),
        }
        .accept_issuance(NonZeroU8::MAX, &trust_anchors, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::MdocPreviewWithSdJwtVcTypeMetadata);
    }

    #[test]
    fn test_accept_issuance_error_metadata_integrity_inconsistent() {
        let (mut signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id_1".to_string().into(), Format::SdJwt)]),
            SdJwtMetadataUsage::TypeMetadata,
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
            session_state: new_session_state(previews, metadata, 4.try_into().unwrap(), true, true),
        }
        .accept_issuance(NonZeroU8::MAX, &trust_anchors, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::MetadataIntegrityInconsistent);
    }

    #[test]
    fn test_accept_issuance_incorrect_resource_integrity() {
        // Only an SD-JWT binds to a Type Metadata integrity digest, so only that format can mismatch it.
        let (mut signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string().into(), Format::SdJwt)]),
            SdJwtMetadataUsage::TypeMetadata,
        );
        let trust_anchors = signer.trust_anchors.clone();

        // Include a random resource integrity in the returned SD-JWT.
        signer.metadata_integrity = Integrity::from(crypto::utils::random_bytes(32));

        let mut mock_msg_client = mock_openid_message_client_nonce(None, 1);

        mock_msg_client.expect_request_credential().return_once(
            move |_url, credential_request, _dpop_header, _access_token_header| {
                Ok(signer.response_from_request(credential_request))
            },
        );

        let error = HttpIssuanceSession {
            message_client: mock_msg_client,
            session_state: new_session_state(previews, metadata, NonZeroU8::MIN, true, true),
        }
        .accept_issuance(NonZeroU8::MAX, &trust_anchors, &MockRemoteWscd::default())
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
        let (signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([("credential_id".to_string().into(), Format::SdJwt)]),
            SdJwtMetadataUsage::TypeMetadata,
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
            session_state: new_session_state(previews, metadata, NonZeroU8::MIN, true, true),
        }
        .accept_issuance(NonZeroU8::MAX, &signer.trust_anchors, &MockRemoteWscd::default())
        .now_or_never()
        .unwrap()
        .expect_err("accepting issuance should not succeed");

        assert_matches!(error, WalletIssuanceError::DeferredIssuanceUnsupported);
    }

    fn mock_credential_response_credential(
        format: Format,
        sd_jwt_uses_credential_metadata: SdJwtMetadataUsage,
    ) -> (
        Credentials,
        PreviewableCredentialPayload,
        OfferedCredentialMetadata,
        PublicKey,
        TrustAnchors,
    ) {
        let credential_id: CredentialId = "credential_id".to_string().into();
        let (signer, previews, metadata) = MockCredentialSigner::new_with_preview_and_type_metadata(
            HashMap::from([(credential_id.clone(), format)]),
            sd_jwt_uses_credential_metadata,
        );

        let holder_pubkey = PublicKey::from(*SigningKey::generate().verifying_key());
        let credentials = signer
            .response_from_holder_pubkeys(&credential_id, vec_nonempty![&holder_pubkey])
            .into_immediate_credentials()
            .unwrap();

        let preview = previews.into_iter().exactly_one().unwrap().credential_payload;
        let metadata = metadata
            .into_iter()
            .exactly_one()
            .expect("a single format is described by a single credential configuration")
            .1;

        (credentials, preview, metadata, holder_pubkey, signer.trust_anchors)
    }

    /// Convert issued credentials using the provided metadata and, if the issuer provided one, preview.
    fn test_convert_credentials_into_issued_credential(
        credentials: Credentials,
        key_identifiers_and_public_keys: VecNonEmpty<(String, PublicKey)>,
        metadata: &OfferedCredentialMetadata,
        preview: Option<&PreviewableCredentialPayload>,
        trust_anchors: &TrustAnchors,
    ) -> Result<(), WalletIssuanceError> {
        match (&credentials, metadata) {
            (Credentials::MsoMdoc(_), OfferedCredentialMetadata::CredentialMetadata(credential_metadata)) => {
                credentials
                    .into_issued_mdocs(
                        key_identifiers_and_public_keys,
                        preview,
                        credential_metadata,
                        trust_anchors,
                    )
                    .map(|_| ())
            }
            (Credentials::SdJwt(_), metadata) => credentials
                .into_issued_sd_jwts(key_identifiers_and_public_keys, metadata, preview, trust_anchors)
                .map(|_| ()),
            _ => panic!("illegal credential format and metadata combination"),
        }
    }

    #[rstest]
    fn test_credential_response_into_credential(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
        #[values(true, false)] has_preview: bool,
    ) {
        let (credentials, preview_data, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format, SdJwtMetadataUsage::TypeMetadata);

        test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            has_preview.then_some(&preview_data),
            &trust_anchor,
        )
        .expect("should be able to convert CredentialResponse into Mdoc");
    }

    #[rstest]
    fn test_credential_response_into_credential_with_sd_jwt_credential_metadata_fallback(
        #[values(true, false)] has_preview: bool,
    ) {
        let (credentials, preview_data, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(Format::SdJwt, SdJwtMetadataUsage::CredentialMetadata);

        assert_matches!(metadata, OfferedCredentialMetadata::CredentialMetadata(_));

        test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            has_preview.then_some(&preview_data),
            &trust_anchor,
        )
        .expect("should be able to convert CredentialResponse into SD-JWT described by Credential Metadata");
    }

    #[rstest]
    fn test_credential_response_into_mdoc_attribute_random_length_error(#[values(true, false)] has_preview: bool) {
        let (credentials, preview_data, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(Format::MsoMdoc, SdJwtMetadataUsage::CredentialMetadata);

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

        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            has_preview.then_some(&preview_data),
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::AttributeRandomLength(5, ATTR_RANDOM_LENGTH));
    }

    #[rstest]
    fn test_credential_response_into_sd_jwt_sd_jwt_verification_error(#[values(true, false)] has_preview: bool) {
        let (credentials, preview, metadata, holder_public_key, _) =
            mock_credential_response_credential(Format::SdJwt, SdJwtMetadataUsage::TypeMetadata);

        // Converting a `CredentialResponse` into an SD-JWT credential that
        // is validated against incorrect trust anchors should fail.
        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            has_preview.then_some(&preview),
            &TrustAnchors::empty(),
        )
        .expect_err("should not be able to convert CredentialResponse into SD-JWT");

        assert_matches!(error, WalletIssuanceError::SdJwtVerification(_));
    }

    #[rstest]
    fn test_credential_response_into_mdoc_public_key_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
        #[values(true, false)] has_preview: bool,
    ) {
        let (credentials, preview_data, metadata, _, trust_anchor) =
            mock_credential_response_credential(format, SdJwtMetadataUsage::TypeMetadata);

        // Converting a `CredentialResponse` into an `Mdoc` using a different mdoc
        // public key than the one contained within the response should fail.
        let other_public_key = PublicKey::from(*SigningKey::generate().verifying_key());
        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), other_public_key)],
            &metadata,
            has_preview.then_some(&preview_data),
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into credential");

        assert_matches!(error, WalletIssuanceError::PublicKeyMismatch);
    }

    #[rstest]
    fn test_credential_response_into_mdoc_mdoc_verification_error(#[values(true, false)] has_preview: bool) {
        let (credentials, preview, metadata, holder_public_key, _) =
            mock_credential_response_credential(Format::MsoMdoc, SdJwtMetadataUsage::CredentialMetadata);

        // Converting a `CredentialResponse` into an `Mdoc` that is
        // validated against incorrect trust anchors should fail.
        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            has_preview.then_some(&preview),
            &TrustAnchors::empty(),
        )
        .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::MdocVerification(_));
    }

    #[test]
    fn test_credential_response_into_mdoc_issued_attributes_mismatch_error() {
        let (credentials, mut preview, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(Format::MsoMdoc, SdJwtMetadataUsage::CredentialMetadata);

        // Converting a `CredentialResponse` into an `Mdoc` with different attributes
        // in the preview than are contained within the response should fail. Note that these stay laid out in the
        // mdoc name space, so that the extra attribute is the only difference.
        preview.attributes = Attributes::example([
            ([PID_ATTESTATION_TYPE, "new"], Attribute::Bool(true)),
            (
                [PID_ATTESTATION_TYPE, "family_name"],
                Attribute::Text(String::from("De Bruijn")),
            ),
        ]);

        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            Some(&preview),
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_doctype_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format, SdJwtMetadataUsage::TypeMetadata);

        // Converting a `CredentialResponse` into an `Mdoc` with a different doc_type in the preview than contained
        // within the response should fail.
        preview.attestation_type = String::from("other.attestation_type");

        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            Some(&preview),
            &trust_anchor,
        )
        .expect_err("should not be able to convert CredentialResponse into Mdoc");

        assert_matches!(error, WalletIssuanceError::IssuedCredentialMismatch { .. });
    }

    #[rstest]
    fn test_credential_response_into_mdoc_issued_validity_info_mismatch_error(
        #[values(Format::MsoMdoc, Format::SdJwt)] format: Format,
    ) {
        let (credentials, mut preview, metadata, holder_public_key, trust_anchor) =
            mock_credential_response_credential(format, SdJwtMetadataUsage::TypeMetadata);

        // Converting a `CredentialResponse` into an `Mdoc` with different expiration information in the preview than
        // contained within the response should fail.

        preview.not_before = Some((Utc::now() + chrono::Duration::days(1)).into());

        let error = test_convert_credentials_into_issued_credential(
            credentials,
            vec_nonempty![("key_id".to_string(), holder_public_key)],
            &metadata,
            Some(&preview),
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
