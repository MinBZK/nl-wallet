use chrono::{DateTime, Utc};
use nutype::nutype;
use webpki::{
    EndEntityCert, KeyUsage, Time, TrustAnchor, ECDSA_P256_SHA256, ECDSA_P256_SHA384, ECDSA_P384_SHA256,
    ECDSA_P384_SHA384,
};
use x509_parser::{certificate::X509Certificate, error::X509Error, prelude::FromDer};

#[derive(Debug, thiserror::Error)]
pub enum CertificateChainError {
    #[error("parsing or verification failed: {0}")]
    CertificateChain(#[from] webpki::Error),
    #[error("provided time is earlier than the unix epoch")]
    TimeBeforeUnixEpoch,
}

#[nutype(
    validate(predicate = |certs| !certs.is_empty()),
    derive(Debug, Clone, TryFrom, AsRef)
)]
pub struct DerX509CertificateChain(Vec<Vec<u8>>);

impl DerX509CertificateChain {
    fn credential_certificate_der(&self) -> &[u8] {
        self.as_ref().first().unwrap()
    }

    fn intermediate_certificates_der(&self) -> Vec<&[u8]> {
        self.as_ref().iter().skip(1).map(Vec::as_slice).collect()
    }

    pub fn credential_certificate(&self) -> Result<X509Certificate, X509Error> {
        let (_, cert) = X509Certificate::from_der(self.credential_certificate_der())?;

        Ok(cert)
    }

    pub(crate) fn verify(
        &self,
        trust_anchors: &[TrustAnchor],
        time: DateTime<Utc>,
    ) -> Result<(), CertificateChainError> {
        let timestamp = time
            .timestamp()
            .try_into()
            .map_err(|_| CertificateChainError::TimeBeforeUnixEpoch)?;

        EndEntityCert::try_from(self.credential_certificate_der())?.verify_for_usage(
            &[
                &ECDSA_P256_SHA256,
                &ECDSA_P256_SHA384,
                &ECDSA_P384_SHA256,
                &ECDSA_P384_SHA384,
            ],
            trust_anchors,
            &self.intermediate_certificates_der(),
            Time::from_seconds_since_unix_epoch(timestamp),
            KeyUsage::client_auth(),
            &[],
        )?;

        Ok(())
    }
}
