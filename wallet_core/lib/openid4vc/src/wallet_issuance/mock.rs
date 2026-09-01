use std::cell::RefCell;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use crypto::trust_anchor::TrustAnchors;
use derive_more::From;
use jwt::nonce::Nonce;
use jwt::wia::WiaDisclosure;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use wscd::mock_remote::MockWiaClient;
use wscd::wscd::WiaClient;

use super::AuthorizationSession;
use super::CredentialSelection;
use super::IssuanceDiscovery;
use super::IssuanceDiscoveryParameters;
use super::IssuanceFlow;
use super::IssuanceSession;
use super::WalletIssuanceError;
use super::credential::CredentialWithMetadata;
use crate::token::CredentialPreview;

/// A [`WiaClient`] that records the challenge it was given, delegating the actual WIA issuance to a
/// [`MockWiaClient`].
#[derive(Default)]
pub struct RecordingWiaClient {
    pub received_challenge: RefCell<Option<Option<Nonce>>>,
}

impl WiaClient for RecordingWiaClient {
    type Error = <MockWiaClient as WiaClient>::Error;

    async fn issue_wia(&self, aud: String, challenge: Option<Nonce>) -> Result<WiaDisclosure, Self::Error> {
        *self.received_challenge.borrow_mut() = Some(challenge.clone());
        MockWiaClient::new().issue_wia(aud, challenge).await
    }
}

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
            selection: &CredentialSelection,
        ) -> Result<IssuanceFlow<MockAuthorizationSession, MockIssuanceSession>, WalletIssuanceError>;

        pub fn start_authorization_code_flow_sync(&self, selection: &CredentialSelection) -> Result<MockAuthorizationSession, WalletIssuanceError>;

        pub fn start_pre_authorized_code_flow_sync(&self, selection: &CredentialSelection) -> Result<MockIssuanceSession, WalletIssuanceError>;

        pub fn restore_authorization_session_sync(&self, data: MockAuthorizationSessionData) -> MockAuthorizationSession;
    }
}

impl IssuanceDiscovery for MockIssuanceDiscovery {
    type Authorization = MockAuthorizationSession;
    type Issuance = MockIssuanceSession;

    async fn start<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
        _client_id: String,
        _redirect_uri: Url,
        _issuer_trust_anchors: &TrustAnchors,
    ) -> Result<IssuanceFlow<Self::Authorization, Self::Issuance>, WalletIssuanceError>
    where
        W: WiaClient,
    {
        self.start_sync(common_parameters.selection)
    }

    async fn start_authorization_code_flow<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
        _client_id: String,
        _redirect_uri: Url,
    ) -> Result<Self::Authorization, WalletIssuanceError>
    where
        W: WiaClient,
    {
        self.start_authorization_code_flow_sync(common_parameters.selection)
    }

    async fn start_pre_authorized_code_flow<'a, W>(
        &self,
        common_parameters: IssuanceDiscoveryParameters<'a, W>,
        _issuer_trust_anchors: &TrustAnchors,
    ) -> Result<Self::Issuance, WalletIssuanceError>
    where
        W: WiaClient,
    {
        self.start_pre_authorized_code_flow_sync(common_parameters.selection)
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
        _wia_client: &impl WiaClient,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        self.start_issuance_sync()
    }
}

/// Helper type that allows `mockall` to return references from a mocked method.
#[derive(From)]
pub struct MockIssuanceSessionPreviewsWithMetadata(Vec<(CredentialPreview, NormalizedTypeMetadata)>);

mockall::mock! {
    #[derive(Debug)]
    pub IssuanceSession {
        pub fn accept(&self) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError>;

        pub fn previews_with_metadata(&self) -> &MockIssuanceSessionPreviewsWithMetadata;

        pub fn issuer(&self) -> &IssuerRegistration;
    }
}

impl IssuanceSession for MockIssuanceSession {
    async fn accept_issuance<W>(
        &mut self,
        _: &TrustAnchors,
        _: &W,
    ) -> Result<Vec<CredentialWithMetadata>, WalletIssuanceError> {
        self.accept()
    }

    fn previews_with_metadata(&self) -> impl Iterator<Item = (&CredentialPreview, &NormalizedTypeMetadata)> {
        let MockIssuanceSessionPreviewsWithMetadata(inner) = self.previews_with_metadata();

        inner.iter().map(|(preview, metadata)| (preview, metadata))
    }

    fn issuer_registration(&self) -> &IssuerRegistration {
        self.issuer()
    }
}
