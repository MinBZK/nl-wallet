use chrono::DateTime;
use chrono::Utc;
use derive_more::derive::AsRef;
use p256::ecdsa::VerifyingKey;
use passkey_types::ctap2::Aaguid;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::TryFromInto;
use sha2::Digest;
use sha2::Sha256;

use crate::app_identifier::AppIdentifier;
use crate::auth_data::FullAuthenticatorDataWithSource;
use crate::certificates::CertificateError;
use crate::certificates::DerX509CertificateChain;

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
    #[error("initial counter is not present in authenticator data")]
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
    pub fn aaguid(&self) -> Aaguid {
        let guid = match self {
            Self::Development => b"appattestdevelop",
            Self::Production => b"appattest\0\0\0\0\0\0\0",
        };

        Aaguid::from(*guid)
    }
}

#[derive(Debug, Clone, AsRef)]
pub struct VerifiedAttestation(Attestation);

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    #[serde(rename = "fmt")]
    pub format: AttestationFormat,
    #[serde(rename = "attStmt")]
    pub attestation_statement: AttestationStatement,
    #[serde_as(as = "TryFromInto<Vec<u8>>")]
    pub auth_data: FullAuthenticatorDataWithSource,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[serde(rename_all = "kebab-case")]
pub enum AttestationFormat {
    #[default]
    AppleAppattest,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
pub struct AttestationStatement {
    #[serde_as(as = "TryFromInto<Vec<Vec<u8>>>")]
    #[serde(rename = "x5c")]
    pub x509_certificates: DerX509CertificateChain,
    pub receipt: Vec<u8>,
}

impl Attestation {
    pub fn parse(bytes: &[u8]) -> Result<Self, AttestationError> {
        let attestation = ciborium::from_reader(bytes).map_err(AttestationDecodingError::Cbor)?;

        Ok(attestation)
    }
}

impl VerifiedAttestation {
    pub fn parse_and_verify(
        bytes: &[u8],
        trust_anchors: &[TrustAnchor],
        challenge: &[u8],
        app_identifier: &AppIdentifier,
        environment: AttestationEnvironment,
    ) -> Result<(Self, VerifyingKey), AttestationError> {
        Self::parse_and_verify_with_time(bytes, trust_anchors, Utc::now(), challenge, app_identifier, environment)
    }

    pub fn parse_and_verify_with_time(
        bytes: &[u8],
        trust_anchors: &[TrustAnchor],
        time: DateTime<Utc>,
        challenge: &[u8],
        app_identifier: &AppIdentifier,
        environment: AttestationEnvironment,
    ) -> Result<(Self, VerifyingKey), AttestationError> {
        let attestation = Attestation::parse(bytes)?;

        // The steps below are listed at:
        // https://developer.apple.com/documentation/devicecheck/validating-apps-that-connect-to-your-server#Verify-the-attestation

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

        let extension = credential_certificate
            .attestation_extension()
            .map_err(AttestationDecodingError::CertificateExtension)?;

        if *nonce != *extension.nonce {
            return Err(AttestationValidationError::NonceMismatch)?;
        }

        // 5. Create the SHA256 hash of the public key in credCert with X9.62 uncompressed point format, and verify that
        //    it matches the key identifier from your app.

        // NB: The Apple documentation specifies that the app should send the key identifier to the server, along with
        //     the attestation data. However, this does not appear to add any additional guarantees, as the key
        //     identifier calculated here is already compared with the value extracted in step 9. For this reason we
        //     do not take the key identifier as a parameter and only calculate it here in preparation for step 9.

        let key_identifier = Sha256::digest(public_key.to_encoded_point(false));

        // 6. Compute the SHA256 hash of your app’s App ID, and verify that it’s the same as the authenticator data’s RP
        //    ID hash.

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
        //    development environment, or appattest followed by seven 0x00 bytes if operating in the production
        //    environment.

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

        Ok((VerifiedAttestation(attestation), public_key))
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use coset::iana::EllipticCurve;
    use coset::CoseKeyBuilder;
    use der::Encode;
    use derive_more::Debug;
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::DecodePrivateKey;
    use passkey_types::ctap2::AttestedCredentialData;
    use passkey_types::ctap2::AuthenticatorData;
    use rand::RngCore;
    use rcgen::BasicConstraints;
    use rcgen::Certificate;
    use rcgen::CertificateParams;
    use rcgen::CustomExtension;
    use rcgen::IsCa;
    use rcgen::KeyPair;
    use rcgen::PKCS_ECDSA_P256_SHA256;
    use rcgen::PKCS_ECDSA_P384_SHA384;
    use rustls_pki_types::TrustAnchor;
    use sha2::Digest;
    use sha2::Sha256;

    use crate::app_identifier::AppIdentifier;
    use crate::auth_data::FullAuthenticatorDataWithSource;
    use crate::certificates::AppleAnonymousAttestationExtension;
    use crate::certificates::APPLE_ANONYMOUS_ATTESTATION_OID;

    use super::Attestation;
    use super::AttestationEnvironment;
    use super::AttestationFormat;
    use super::AttestationStatement;

    #[derive(Debug)]
    pub struct MockAttestationCa {
        #[debug("{:?}", certificate.der())]
        certificate: Certificate,
        key_pair: KeyPair,
    }

    impl MockAttestationCa {
        pub fn generate() -> Self {
            let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P384_SHA384).unwrap();

            let mut params = CertificateParams::default();
            params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            let certificate = params.self_signed(&key_pair).unwrap();

            Self { certificate, key_pair }
        }

        pub fn trust_anchor(&self) -> TrustAnchor {
            webpki::anchor_from_trusted_cert(self.certificate.der()).unwrap()
        }
    }

    impl AsRef<[u8]> for MockAttestationCa {
        fn as_ref(&self) -> &[u8] {
            self.certificate.der().as_ref()
        }
    }

    impl Attestation {
        pub fn new_mock(
            ca: &MockAttestationCa,
            challenge: &[u8],
            app_identifier: &AppIdentifier,
        ) -> (Self, SigningKey) {
            // Generate an ECDSA key pair and get both the private and public key from it.
            let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
            let signing_key = SigningKey::from_pkcs8_der(key_pair.serialized_der()).unwrap();

            // Build `AuthenticatorData` with the following contents:
            // * The app identifier string as RP ID.
            // * An initial assertion counter of 0.
            // * The Apple development environment as AAGUID.
            // * A hash of the raw public key as credential ID.
            // * The public key in COSE key format.
            let mut auth_data = AuthenticatorData::new(app_identifier.as_ref(), Some(0));

            let aaguid = AttestationEnvironment::Development.aaguid();

            let verifying_key = signing_key.verifying_key();
            let encoded_point = verifying_key.to_encoded_point(false);
            let credential_id = Sha256::digest(encoded_point).to_vec();

            let key = CoseKeyBuilder::new_ec2_pub_key(
                EllipticCurve::P_256,
                encoded_point.x().unwrap().to_vec(),
                encoded_point.y().unwrap().to_vec(),
            )
            .build();

            auth_data.attested_credential_data = Some(AttestedCredentialData::new(aaguid, credential_id, key).unwrap());

            let auth_data = FullAuthenticatorDataWithSource::from(auth_data);

            // Calculate the nonce based on the serialized `AuthenticatorData` and the provided challenge,
            // then encode this in an Apple anonymous attestation extension. Use this and the key pair to
            // generate a X.509 certiticate.
            let nonce = Sha256::new()
                .chain_update(auth_data.source())
                .chain_update(challenge)
                .finalize()
                .to_vec();
            let extension = AppleAnonymousAttestationExtension { nonce: &nonce };
            let mut extension_content = Vec::new();
            extension.encode_to_vec(&mut extension_content).unwrap();

            let mut params = CertificateParams::default();
            params.custom_extensions = vec![CustomExtension::from_oid_content(
                &APPLE_ANONYMOUS_ATTESTATION_OID,
                extension_content,
            )];
            let certificate = params.signed_by(&key_pair, &ca.certificate, &ca.key_pair).unwrap();

            // Sign the X.509 certificate with the CA private key, then serialize
            // this and the CA certificate into a DER certificate chain.
            let x509_certificates = vec![certificate.der().to_vec(), ca.certificate.der().to_vec()]
                .try_into()
                .unwrap();

            // Generate random receipt data, as this is not validated.
            let mut receipt = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut receipt);

            // Combine all of the above elements into an `Attestation` struct
            // and return it together with the private key.
            let attestation_statement = AttestationStatement {
                x509_certificates,
                receipt,
            };

            let attestation = Attestation {
                format: AttestationFormat::default(),
                attestation_statement,
                auth_data,
            };

            (attestation, signing_key)
        }

        pub fn new_mock_bytes(
            ca: &MockAttestationCa,
            challenge: &[u8],
            app_identifier: &AppIdentifier,
        ) -> (Vec<u8>, SigningKey) {
            let (attestation, signing_key) = Self::new_mock(ca, challenge, app_identifier);

            let mut bytes = Vec::new();
            ciborium::into_writer(&attestation, &mut bytes).unwrap();

            (bytes, signing_key)
        }
    }
}
