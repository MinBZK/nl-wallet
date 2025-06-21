use attestation_data::auth::reader_auth::ReaderRegistration;
use crypto::x509::BorrowingCertificate;

#[derive(Debug, Clone)]
pub struct VerifierCertificate {
    certificate: BorrowingCertificate,
    registration: ReaderRegistration,
}

impl VerifierCertificate {
    pub(super) fn new(certificate: BorrowingCertificate, registration: ReaderRegistration) -> Self {
        Self {
            certificate,
            registration,
        }
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
