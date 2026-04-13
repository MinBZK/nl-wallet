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

        let (issuer_metadata, oauth_metadata) = self.fetch_metadata(&credential_offer.credential_issuer).await?;

        let pre_authorized_code = credential_offer
            .authorization_code()
            .ok_or(WalletIssuanceError::MissingPreAuthorizedCodeGrant)?;

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
