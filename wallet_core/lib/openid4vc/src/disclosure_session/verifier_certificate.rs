use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::x509::CertificateType;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;

#[derive(Debug, Clone)]
pub struct VerifierCertificate {
    certificate: BorrowingCertificate,
    registration: ReaderRegistration,
}

impl VerifierCertificate {
    pub fn try_new(certificate: BorrowingCertificate) -> Result<Option<Self>, CertificateError> {
        let verifier_certificate = match CertificateType::from_certificate(&certificate)? {
            CertificateType::ReaderAuth(Some(reader_registration)) => Some(Self {
                certificate,
                registration: *reader_registration,
            }),
            _ => None,
        };

        Ok(verifier_certificate)
    }

    pub fn certificate(&self) -> &BorrowingCertificate {
        &self.certificate
    }

    pub fn registration(&self) -> &ReaderRegistration {
        &self.registration
    }

    pub fn into_certificate_and_registration(self) -> (BorrowingCertificate, ReaderRegistration) {
        (self.certificate, self.registration)
    }
}
