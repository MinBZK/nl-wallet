use std::hash::Hash;

use rustls_pki_types::TrustAnchor;

use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;
use mdoc::holder::disclosure::PartialMdoc;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;

use crate::verifier::SessionType;

use super::DisclosureClient;
use super::DisclosureSession;
use super::VerifierCertificate;
use super::error::DisclosureError;
use super::error::VpSessionError;
use super::uri_source::DisclosureUriSource;

mockall::mock! {
    #[derive(Debug)]
    pub DisclosureClient {}

    impl DisclosureClient for DisclosureClient {
        type Session = MockDisclosureSession;

        async fn start<'a>(
            &self,
            request_uri_query: &str,
            uri_source: DisclosureUriSource,
            trust_anchors: &[TrustAnchor<'a>],
        ) -> Result<MockDisclosureSession, VpSessionError>;
    }
}

mockall::mock! {
    #[derive(Debug)]
    pub DisclosureSession {
        pub fn session_type(&self) -> SessionType;
        pub fn credential_requests(&self) -> &NormalizedCredentialRequests;
        pub fn verifier_certificate(&self) -> &VerifierCertificate;

        pub async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
        pub async fn disclose(
            self,
            partial_mdocs: VecNonEmpty<PartialMdoc>,
        ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>;
    }
}

impl DisclosureSession for MockDisclosureSession {
    fn session_type(&self) -> SessionType {
        self.session_type()
    }

    fn credential_requests(&self) -> &NormalizedCredentialRequests {
        self.credential_requests()
    }

    fn verifier_certificate(&self) -> &VerifierCertificate {
        self.verifier_certificate()
    }

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        self.terminate().await
    }

    async fn disclose<K, W>(
        self,
        partial_mdocs: VecNonEmpty<PartialMdoc>,
        _: &W,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K, Poa = Poa>,
    {
        self.disclose(partial_mdocs).await
    }
}
