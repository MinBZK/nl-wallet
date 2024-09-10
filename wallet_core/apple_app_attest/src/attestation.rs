use chrono::{DateTime, Utc};
use p256::{ecdsa::VerifyingKey, pkcs8::DecodePublicKey};
use passkey_types::ctap2::AuthenticatorData;
use serde::Deserialize;
use serde_with::{serde_as, TryFromInto};
use webpki::TrustAnchor;
use x509_parser::error::X509Error;

use crate::certificate_chain::{CertificateChainError, DerX509CertificateChain};

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("attestation could not be decoded: {0}")]
    Decoding(#[from] AttestationDecodingError),
    #[error("attestation did not validate: {0}")]
    Validation(#[from] AttestationValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationDecodingError {
    #[error("deserializing CBOR deserialization failed: {0}")]
    Cbor(#[source] ciborium::de::Error<std::io::Error>),
    #[error("decoding X.509 certificate failed: {0}")]
    Certificate(#[source] X509Error),
    #[error("decoding public key failed: {0}")]
    PublicKey(#[source] p256::pkcs8::spki::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationValidationError {
    #[error("certificate chain validation failed: {0}")]
    CertificateChain(#[source] CertificateChainError),
    #[error("intial counter is not present in authenticator data")]
    CounterMissing,
    #[error("counter is not 0, received: {0}")]
    CounterNotZero(u32),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    #[serde(rename = "fmt")]
    pub format: AttestationFormat,
    #[serde(rename = "attStmt")]
    pub attestation_statement: AttestationStatement,
    pub auth_data: AuthenticatorData,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttestationFormat {
    #[default]
    AppleAppattest,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationStatement {
    #[serde_as(as = "TryFromInto<Vec<Vec<u8>>>")]
    #[serde(rename = "x5c")]
    pub x509_certificates: DerX509CertificateChain,
    pub receipt: Vec<u8>,
}

impl Attestation {
    pub fn parse_and_verify(
        bytes: &[u8],
        trust_anchors: &[TrustAnchor],
        time: DateTime<Utc>,
    ) -> Result<(Self, VerifyingKey, u32), AttestationError> {
        let attestation: Self = ciborium::from_reader(bytes).map_err(AttestationDecodingError::Cbor)?;

        // The steps below are listed at:
        // https://developer.apple.com/documentation/devicecheck/validating-apps-that-connect-to-your-server.

        // 1. Verify that the x5c array contains the intermediate and leaf certificates for App Attest, starting from
        //    the credential certificate in the first data buffer in the array (credcert). Verify the validity of the
        //    certificates using Apple’s App Attest root certificate.
        attestation
            .attestation_statement
            .x509_certificates
            .verify(trust_anchors, time)
            .map_err(AttestationValidationError::CertificateChain)?;

        // Extract the public key from the leaf certificate.
        let public_key = VerifyingKey::from_public_key_der(
            attestation
                .attestation_statement
                .x509_certificates
                .credential_certificate()
                .map_err(AttestationDecodingError::Certificate)?
                .public_key()
                .raw,
        )
        .map_err(AttestationDecodingError::PublicKey)?;

        // TODO: Perform steps 2 through 6.

        // 7. Verify that the authenticator data’s counter field equals 0.

        let counter = attestation
            .auth_data
            .counter
            .ok_or(AttestationValidationError::CounterMissing)?;

        if counter != 0 {
            return Err(AttestationValidationError::CounterNotZero(counter))?;
        }

        // TODO: Perform steps 8 and 9.

        Ok((attestation, public_key, counter))
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use const_decoder::{Decoder, Pem};
    use webpki::TrustAnchor;

    use super::Attestation;

    // Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
    const APPLE_ROOT_CA: [u8; 549] = Pem::decode(include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

    // Source: https://developer.apple.com/documentation/devicecheck/attestation-object-validation-guide
    const TEST_ATTESTATION: [u8; 5637] =
        Decoder::Base64.decode(include_bytes!("../assets/test_attestation_object.b64"));
    const TEST_ATTESTATION_VALID_DATE: &str = "2024-04-18T12:00:00Z";

    #[test]
    fn test_attestation() {
        let trust_anchor = TrustAnchor::try_from_cert_der(&APPLE_ROOT_CA).unwrap();
        let time = DateTime::parse_from_rfc3339(TEST_ATTESTATION_VALID_DATE)
            .unwrap()
            .to_utc();

        let _ = Attestation::parse_and_verify(&TEST_ATTESTATION, &[trust_anchor], time)
            .expect("should decode attestation object");
    }
}
