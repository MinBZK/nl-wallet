use chrono::{DateTime, Utc};
use p256::ecdsa::VerifyingKey;
use passkey_types::ctap2::Aaguid;
use serde::Deserialize;
use serde_with::{serde_as, TryFromInto};
use sha2::{Digest, Sha256};
use webpki::TrustAnchor;

use crate::{
    app_identifier::AppIdentifier,
    auth_data::AuthenticatorDataWithSource,
    certificates::{CertificateError, DerX509CertificateChain},
};

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("attestation could not be decoded: {0}")]
    Decoding(#[from] AttestationDecodingError),
    #[error("attestation did not validate: {0}")]
    Validation(#[from] AttestationValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationDecodingError {
    #[error("deserializing attestation CBOR failed: {0}")]
    Cbor(#[source] ciborium::de::Error<std::io::Error>),
    #[error("decoding credential certificate failed: {0}")]
    CredentialCertificate(#[source] CertificateError),
    #[error("decoding public key failed: {0}")]
    PublicKey(#[source] CertificateError),
    #[error("decoding certificate extension data failed: {0}")]
    CertificateExtension(#[source] CertificateError),
    #[error("intial counter is not present in authenticator data")]
    CounterMissing,
    #[error("attested credential data is not present in authenticator data")]
    AttestedCredentialDataMissing,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationValidationError {
    #[error("certificate chain parsing or validation failed: {0}")]
    CertificateChain(#[source] CertificateError),
    #[error("nonce does not match calculated nonce")]
    NonceMismatch,
    #[error("relying party identifier does not match calculated value")]
    RpIdMismatch,
    #[error("counter is not 0, received: {0}")]
    CounterNotZero(u32),
    #[error("attestation environment is not match, expected: {:?}, received: {:?}", expected.0, received.0)]
    EnvironmentMismatch { expected: Aaguid, received: Aaguid },
    #[error("key identifier does not match calculated value")]
    KeyIdentifierMismatch,
}

#[derive(Debug, Clone, Copy)]
pub enum AttestationEnvironment {
    Development,
    Production,
}

impl AttestationEnvironment {
    pub(crate) fn aaguid(&self) -> Aaguid {
        let guid = match self {
            Self::Development => b"appattestdevelop",
            Self::Production => b"appattest\0\0\0\0\0\0\0",
        };

        Aaguid::from(*guid)
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    #[serde(rename = "fmt")]
    pub format: AttestationFormat,
    #[serde(rename = "attStmt")]
    pub attestation_statement: AttestationStatement,
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub auth_data: AuthenticatorDataWithSource,
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
        challenge: &[u8],
        app_identifier: &AppIdentifier,
        environment: AttestationEnvironment,
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
        let credential_certificate = attestation
            .attestation_statement
            .x509_certificates
            .credential_certificate()
            .map_err(AttestationDecodingError::CredentialCertificate)?;
        let public_key = credential_certificate
            .public_key()
            .map_err(AttestationDecodingError::PublicKey)?;

        // 2. Create clientDataHash as the SHA256 hash of the one-time challenge your server sends to your app before
        //    performing the attestation, and append that hash to the end of the authenticator data (authData from the
        //    decoded object).
        // 3. Generate a new SHA256 hash of the composite item to create nonce.

        let nonce = Sha256::new()
            .chain_update(attestation.auth_data.source())
            .chain_update(challenge)
            .finalize();

        // 4. Obtain the value of the credCert extension with OID 1.2.840.113635.100.8.2, which is a DER-encoded ASN.1
        //    sequence. Decode the sequence and extract the single octet string that it contains. Verify that the string
        //    equals nonce.

        let extension_nonce = credential_certificate
            .attestation_extension_data()
            .map_err(AttestationDecodingError::CertificateExtension)?;

        if *nonce != *extension_nonce {
            return Err(AttestationValidationError::NonceMismatch)?;
        }

        // 5. Create the SHA256 hash of the public key in credCert with X9.62 uncompressed point format, and verify that
        // it matches the key identifier from your app.

        let key_identifier = Sha256::digest(public_key.to_encoded_point(false));

        // 6. Compute the SHA256 hash of your app’s App ID, and verify that it’s the same as the authenticator data’s
        // RP ID hash.

        if attestation.auth_data.as_ref().rp_id_hash() != app_identifier.sha256_hash() {
            return Err(AttestationValidationError::RpIdMismatch)?;
        }

        // 7. Verify that the authenticator data’s counter field equals 0.

        let counter = attestation
            .auth_data
            .as_ref()
            .counter
            .ok_or(AttestationDecodingError::CounterMissing)?;

        if counter != 0 {
            return Err(AttestationValidationError::CounterNotZero(counter))?;
        }

        // 8. Verify that the authenticator data’s aaguid field is either appattestdevelop if operating in the
        // development environment, or appattest followed by seven 0x00 bytes if operating in the production
        // environment.

        let attested_credential_data = attestation
            .auth_data
            .as_ref()
            .attested_credential_data
            .as_ref()
            .ok_or(AttestationDecodingError::AttestedCredentialDataMissing)?;

        let environment_aaguid = environment.aaguid();
        if attested_credential_data.aaguid != environment_aaguid {
            return Err(AttestationValidationError::EnvironmentMismatch {
                expected: environment_aaguid,
                received: attested_credential_data.aaguid,
            })?;
        }

        // 9. Verify that the authenticator data’s credentialId field is the same as the key identifier.

        if *attested_credential_data.credential_id() != *key_identifier {
            return Err(AttestationValidationError::KeyIdentifierMismatch)?;
        }

        Ok((attestation, public_key, counter))
    }
}
