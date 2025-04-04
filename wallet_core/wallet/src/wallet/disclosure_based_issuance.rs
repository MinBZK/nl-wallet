use std::sync::Arc;

use tracing::instrument;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::disclosure_session::VpMessageClientError;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::PostAuthResponseErrorCode;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_common::http::TlsPinningConfig;
use wallet_common::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::attestation::Attestation;
use crate::disclosure::MdocDisclosureSession;
use crate::errors::UpdatePolicyError;
use crate::issuance::DigidSession;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;

use super::disclosure::RedirectUriPurpose;
use super::DisclosureError;
use super::IssuanceError;
use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DisclosureBasedIssuanceError {
    #[error("disclosure failed: {0}")]
    Disclosure(#[from] DisclosureError),
    #[error("retrieving attribute previews failed: {0}")]
    Issuance(#[from] IssuanceError),
    #[error("missing redirect URI from verifier response")]
    #[category(critical)]
    MissingRedirectUri,
    #[error("missing query in redirect URI")]
    #[category(critical)]
    MissingRedirectUriQuery,
    #[error("failed to deserialize Credential Offer: {0}")]
    #[category(pd)]
    UrlDecoding(#[from] serde_urlencoded::de::Error),
    #[error("no grants found in Credential Offer")]
    #[category(critical)]
    MissingGrants,
    #[error("no Authorization Code found in Credential Offer")]
    #[category(critical)]
    MissingAuthorizationCode,
}

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    DS: DigidSession,
    IS: IssuanceSession,
    MDS: MdocDisclosureSession<Self>,
    WIC: Default,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_disclosure_based_issuance(
        &mut self,
        pin: String,
    ) -> Result<Vec<Attestation>, DisclosureBasedIssuanceError> {
        let redirect_uri = match self.perform_disclosure(pin, RedirectUriPurpose::Issuance).await {
            Ok(redirect_uri) => redirect_uri,

            // If the issuer has no attestations to issue, return an empty Vec.
            Err(DisclosureError::VpDisclosureSession(VpClientError::Request(
                VpMessageClientError::AuthPostResponse(err),
            ))) if err.error_response.error == PostAuthResponseErrorCode::NoIssuableAttestations => {
                return Ok(vec![]);
            }

            Err(err) => Err(err)?,
        };

        let query = redirect_uri
            .as_ref()
            .ok_or(DisclosureBasedIssuanceError::MissingRedirectUri)?
            .query()
            .ok_or(DisclosureBasedIssuanceError::MissingRedirectUriQuery)?;

        let CredentialOfferContainer { credential_offer } = serde_urlencoded::from_str(query)?;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: credential_offer
                    .grants
                    .ok_or(DisclosureBasedIssuanceError::MissingGrants)?
                    .authorization_code()
                    .ok_or(DisclosureBasedIssuanceError::MissingAuthorizationCode)?,
            },
            code_verifier: None,
            client_id: None,
            redirect_uri: None,
        };

        let previews = self
            .issuance_fetch_previews(token_request, credential_offer.credential_issuer, false)
            .await?;

        Ok(previews)
    }
}
