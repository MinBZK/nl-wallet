use std::hash::Hash;

use rustls_pki_types::TrustAnchor;

use crypto::factory::KeyFactory;
use crypto::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use mdoc::holder::Mdoc;
use poa::factory::PoaFactory;
use utils::vec_at_least::VecNonEmpty;

use crate::verifier::SessionType;

use super::error::DisclosureError;
use super::error::VpSessionError;
use super::uri_source::DisclosureUriSource;
use super::AttestationAttributePaths;
use super::DisclosureClient;
use super::DisclosureSession;
use super::VerifierCertificate;

mockall::mock! {
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
    pub DisclosureSession {
        fn session_type(&self) -> SessionType;
        fn requested_attribute_paths(&self) -> &AttestationAttributePaths;
        fn verifier_certificate(&self) -> &VerifierCertificate;

        async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError>;
        async fn disclose(
            self,
            mdocs: VecNonEmpty<Mdoc>,
        ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>;
    }
}

impl DisclosureSession for MockDisclosureSession {
    fn session_type(&self) -> SessionType {
        self.session_type()
    }

    fn requested_attribute_paths(&self) -> &AttestationAttributePaths {
        self.requested_attribute_paths()
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
