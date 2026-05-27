use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::trust_anchor::TrustAnchors;
use url::Url;

use super::AuthorizationSession;
use super::IssuanceDiscovery;
use super::IssuanceFlow;
use super::IssuanceSession;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use super::preview::NormalizedCredentialPreview;
use crate::issuer_identifier::IssuerIdentifier;

mockall::mock! {
    #[derive(Debug)]
    pub IssuanceDiscovery {
        pub fn start_authorization_code_flow_sync(&self) -> Result<MockAuthorizationSession, WalletIssuanceError>;
        pub fn start_with_credential_offer_sync(&self) -> Result<IssuanceFlow<MockAuthorizationSession, MockIssuanceSession>, WalletIssuanceError>;
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

    async fn start_with_credential_offer(
        &self,
        _offer_uri: &Url,
        _client_id: String,
        _redirect_uri: Url,
        _issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError> {
        self.start_with_credential_offer_sync()
    }
}

mockall::mock! {
    #[derive(Debug)]
    pub AuthorizationSession {
        pub fn get_auth_url(&self) -> &Url;
        pub fn get_state(&self) -> &str;
        pub fn start_issuance_sync(&self) -> Result<MockIssuanceSession, WalletIssuanceError>;
    }
}

impl AuthorizationSession for MockAuthorizationSession {
    type Issuance = MockIssuanceSession;

    fn auth_url(&self) -> &Url {
        self.get_auth_url()
    }

    fn state(&self) -> &str {
        self.get_state()
    }

    async fn start_issuance(
        self,
        _received_redirect_uri: &Url,
        _trust_anchors: &TrustAnchors,
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
        _: &TrustAnchors,
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
