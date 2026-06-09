use std::collections::HashMap;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::trust_anchor::TrustAnchors;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use super::AuthorizationSession;
use super::IssuanceDiscovery;
use super::IssuanceFlow;
use super::IssuanceSession;
use super::IssuanceTypeMetadata;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::token::CredentialPreview;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAuthorizationSessionData {
    pub auth_url: Url,
    pub state: String,
}

mockall::mock! {
    #[derive(Debug)]
    pub IssuanceDiscovery {
        pub fn start_sync(
            &self,
        ) -> Result<IssuanceFlow<MockAuthorizationSession, MockIssuanceSession>, WalletIssuanceError>;

        pub fn start_authorization_code_flow_sync(&self) -> Result<MockAuthorizationSession, WalletIssuanceError>;

        pub fn start_pre_authorized_code_flow_sync(&self) -> Result<MockIssuanceSession, WalletIssuanceError>;

        pub fn restore_authorization_session_sync(&self, data: MockAuthorizationSessionData) -> MockAuthorizationSession;
    }
}

impl IssuanceDiscovery for MockIssuanceDiscovery {
    type Authorization = MockAuthorizationSession;
    type Issuance = MockIssuanceSession;

    async fn start(
        &self,
        _offer_uri: &Url,
        _client_id: String,
        _redirect_uri: Url,
        _issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError> {
        self.start_sync()
    }

    async fn start_authorization_code_flow(
        &self,
        _offer_uri: &Url,
        _client_id: String,
        _redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError> {
        self.start_authorization_code_flow_sync()
    }

    async fn start_pre_authorized_code_flow(
        &self,
        _offer_uri: &Url,
        _client_id: String,
        _issuer_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        self.start_pre_authorized_code_flow_sync()
    }

    fn restore_authorization_session(
        &self,
        data: <Self::Authorization as AuthorizationSession>::Persisted,
    ) -> Self::Authorization {
        self.restore_authorization_session_sync(data)
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
    type Persisted = MockAuthorizationSessionData;

    fn auth_url(&self) -> &Url {
        self.get_auth_url()
    }

    fn state(&self) -> &str {
        self.get_state()
    }

    fn persist(&self) -> Self::Persisted {
        MockAuthorizationSessionData {
            auth_url: self.get_auth_url().clone(),
            state: self.get_state().to_string(),
        }
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

        pub fn reject(&self) -> Result<(), WalletIssuanceError>;

        pub fn credential_previews(&self) -> &[CredentialPreview];

        pub fn type_metadata(&self) -> &HashMap<CredentialConfigurationId, IssuanceTypeMetadata>;

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

    async fn reject_issuance(&self) -> Result<(), WalletIssuanceError> {
        self.reject()
    }

    fn credential_previews(&self) -> &[CredentialPreview] {
        self.credential_previews()
    }

    fn type_metadata(&self) -> &HashMap<CredentialConfigurationId, IssuanceTypeMetadata> {
        self.type_metadata()
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        self.issuer()
    }
}
