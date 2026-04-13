use rustls_pki_types::TrustAnchor;
use url::Url;

use attestation_data::auth::issuer_auth::IssuerRegistration;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::token::TokenRequest;
use crate::wallet_issuance::AuthorizationSession;
use crate::wallet_issuance::IssuanceDiscovery;
use crate::wallet_issuance::IssuanceSession;
use crate::wallet_issuance::WalletIssuanceError;
use crate::wallet_issuance::credential::CredentialWithMetadata;
use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
use crate::wallet_issuance::issuance_session::VcMessageClient;
use crate::wallet_issuance::preview::NormalizedCredentialPreview;

mockall::mock! {
    #[derive(Debug)]
    pub IssuanceDiscovery {
        pub fn start_authorization_code_flow_sync(&self) -> Result<MockAuthorizationSession, WalletIssuanceError>;
        pub fn start_pre_authorized_code_flow_sync(&self) -> Result<MockIssuanceSession, WalletIssuanceError>;
    }
}

impl IssuanceDiscovery for MockIssuanceDiscovery {
    type Authorization = MockAuthorizationSession;
    type Issuance = MockIssuanceSession;

    async fn start_authorization_code_flow(
        &self,
        _identifier: &IssuerIdentifier,
        _client_id: String,
        _redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError> {
        self.start_authorization_code_flow_sync()
    }

    async fn start_pre_authorized_code_flow(
        &self,
        _redirect_uri: &Url,
        _client_id: String,
        _trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        self.start_pre_authorized_code_flow_sync()
    }
}

mockall::mock! {
    #[derive(Debug)]
    pub AuthorizationSession {
        pub fn get_auth_url(&self) -> &Url;
        pub fn start_issuance_sync(&self) -> Result<MockIssuanceSession, WalletIssuanceError>;
    }
}

impl AuthorizationSession for MockAuthorizationSession {
    type Issuance = MockIssuanceSession;

    fn auth_url(&self) -> &Url {
        self.get_auth_url()
    }

    async fn start_issuance(
        self,
        _received_redirect_uri: &Url,
        _trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        self.start_issuance_sync()
    }
}

mockall::mock! {
    #[derive(Debug)]
    pub IssuanceSession {
        pub fn accept(
            &self,
        ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>;

        pub fn reject(self) -> Result<(), WalletIssuanceError>;

        pub fn normalized_credential_previews(&self) -> &[NormalizedCredentialPreview];

        pub fn issuer(&self) -> &IssuerRegistration;
    }
}

impl IssuanceSession for MockIssuanceSession {
    async fn accept_issuance<W>(
        &mut self,
        _: &[TrustAnchor<'_>],
        _: &W,
        _: bool,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError> {
        self.accept()
    }

    async fn reject_issuance(self) -> Result<(), WalletIssuanceError> {
        self.reject()
    }

    fn normalized_credential_preview(&self) -> &[NormalizedCredentialPreview] {
        self.normalized_credential_previews()
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        self.issuer()
    }
}

impl<H: VcMessageClient> HttpIssuanceSession<H> {
    pub async fn new_mock(
        message_client: H,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        token_request: TokenRequest,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, WalletIssuanceError> {
        Self::create(
            message_client,
            issuer_metadata,
            oauth_metadata,
            token_request,
            trust_anchors,
        )
        .await
    }
}
