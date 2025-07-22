use std::hash::Hash;

use rustls_pki_types::TrustAnchor;

use attestation_types::request::NormalizedCredentialRequest;
use crypto::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use mdoc::holder::Mdoc;
use utils::vec_at_least::VecNonEmpty;
use wscd::factory::PoaFactory;
use wscd::keyfactory::KeyFactory;

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
        pub fn credential_requests(&self) -> &VecNonEmpty<NormalizedCredentialRequest>;
        pub fn verifier_certificate(&self) -> &VerifierCertificate;

        pub async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
        pub async fn disclose(
            self,
            mdocs: VecNonEmpty<Mdoc>,
        ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>;
    }
}

impl DisclosureSession for MockDisclosureSession {
    fn session_type(&self) -> SessionType {
        self.session_type()
    }

    fn credential_requests(&self) -> &VecNonEmpty<NormalizedCredentialRequest> {
        self.credential_requests()
    }

    fn verifier_certificate(&self) -> &VerifierCertificate {
        self.verifier_certificate()
    }

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        self.terminate().await
    }

    async fn disclose<K, KF>(
        self,
        mdocs: VecNonEmpty<Mdoc>,
        _key_factory: &KF,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
    {
        self.disclose(mdocs).await
    }
}
