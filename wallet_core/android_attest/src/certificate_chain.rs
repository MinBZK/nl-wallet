use std::iter;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::UnixTime;
use webpki::ring::ECDSA_P256_SHA256;
use webpki::ring::ECDSA_P256_SHA384;
use webpki::ring::ECDSA_P384_SHA256;
use webpki::ring::ECDSA_P384_SHA384;
use webpki::ring::RSA_PKCS1_2048_8192_SHA256;
use webpki::ring::RSA_PKCS1_2048_8192_SHA384;
use webpki::ring::RSA_PKCS1_2048_8192_SHA512;
use webpki::ring::RSA_PKCS1_3072_8192_SHA384;
use webpki::EndEntityCert;
use webpki::KeyUsage;
use x509_parser::prelude::FromDer;
use x509_parser::prelude::X509Certificate;
use x509_parser::prelude::X509Error;

use crate::android_crl::RevocationStatusList;
use crate::attestation_extension::key_attestation::KeyAttestationVerificationError;
use crate::attestation_extension::KeyAttestationExtension;
use crate::attestation_extension::KeyAttestationExtensionError;
use crate::root_public_key::RootPublicKey;

#[derive(Debug, thiserror::Error)]
pub enum GoogleKeyAttestationError {
    #[error("invalid trust anchor: {0}")]
    InvalidTrustAnchor(#[source] webpki::Error),
    #[error("invalid certificate chain: {0}")]
    InvalidCertificateChain(#[source] webpki::Error),
    #[error("could not decode certificate from chain: {0}")]
    CertificateDecode(#[source] x509_parser::nom::Err<X509Error>),
    #[error("could not decode end entity certificate from leaf certificate: {0}")]
    LeafCertificate(#[source] webpki::Error),
    #[error("could not decode public key from root certificate: {0}")]
    RootPublicKey(#[source] X509Error),
    #[error("root CA in certificate chain does not contain any of the configured public keys")]
    RootPublicKeyMismatch,
    #[error("certificate chain contains at least one revoked certificate: {}", .0.join(" "))]
    RevokedCertificates(Vec<String>),
    #[error("no key attestation extension found in certificate chain")]
    NoKeyAttestationExtension,
    #[error("could not extract key attestation extension: {0}")]
    KeyAttestationExtension(#[from] KeyAttestationExtensionError),
    #[error("key attestation extension does not meet requirements: {0}")]
    KeyAttestationVerification(#[from] KeyAttestationVerificationError),
}

pub fn verify_google_key_attestation<'a>(
    certificate_chain: &'a [CertificateDer],
    root_public_keys: &[RootPublicKey],
    revocation_list: &RevocationStatusList,
    attestation_challenge: &[u8],
) -> Result<X509Certificate<'a>, GoogleKeyAttestationError> {
    verify_google_key_attestation_with_time(
        certificate_chain,
        root_public_keys,
        revocation_list,
        attestation_challenge,
        Utc::now(),
    )
}

// This function implements the steps as described in: [Verify hardware-backed key pairs with key attestation](https://developer.android.com/privacy-and-security/security-key-attestation).
// The first steps in the procedure are executed on the Android device, and are the prerequisite for this function.
//
// Note that this function has two preconditions, either of which will cause a panic if not met:
// * The certificate chain should contain at least two values.
// * The provided time should be equal to or later than the Unix epoch.
//
// The return value of the function is the decoded leaf certificate.
//
// 1. Use a KeyStore object's getCertificateChain() method to get a reference to the chain of X.509 certificates
//    associated with the hardware-backed keystore.
//
// 2. Send the certificates to a separate server that you trust for validation.
pub fn verify_google_key_attestation_with_time<'a>(
    certificate_chain: &'a [CertificateDer],
    root_public_keys: &[RootPublicKey],
    revocation_list: &RevocationStatusList,
    attestation_challenge: &[u8],
    time: DateTime<Utc>,
) -> Result<X509Certificate<'a>, GoogleKeyAttestationError> {
    assert!(certificate_chain.len() >= 2);

    let timestamp = time
        .timestamp()
        .try_into()
        .expect("provided time should be equal to or later than the Unix epoch");
    let unix_time = UnixTime::since_unix_epoch(Duration::from_secs(timestamp));

    // 3. Obtain a reference to the X.509 certificate chain parsing and validation library that is most appropriate for
    //    your toolset. Verify that the root public certificate is trustworthy and that each certificate signs the next
    //    certificate in the chain.
    let root_certificate = verify_google_attestation_certificate_chain(certificate_chain, root_public_keys, unix_time)?;

    // 4. Check each certificate's revocation status to ensure that none of the certificates have been revoked.

    // Create an iterator that decodes all certificates in the reverse chain, except for the root certificate.
    let remaining_certificates = certificate_chain
        .iter()
        .rev()
        .skip(1)
        .map(|der| X509Certificate::from_der(der.as_ref()).map(|(_, cert)| cert));
    // Append that iterator to the root certificate to create a full reverse chain, from root to leaf.
    let mut x509_certificates = iter::once(Ok(root_certificate))
        .chain(remaining_certificates)
        .collect::<Result<Vec<_>, _>>()
        .map_err(GoogleKeyAttestationError::CertificateDecode)?;

    let revocation_log = revocation_list
        .get_revoked_certificates(&x509_certificates)
        .into_iter()
        .map(|(cert, reason)| {
            format!(
                "subject: {}, serial: {}, status: {:?}",
                cert.subject,
                cert.raw_serial_as_string(),
                reason
            )
        })
        .collect::<Vec<_>>();

    if !revocation_log.is_empty() {
        return Err(GoogleKeyAttestationError::RevokedCertificates(revocation_log));
    }

    // 5. Optionally, inspect the provisioning information certificate extension that is only present in newer
    //    certificate chains.
    //
    // We skip this step, as the provisioning information only contains a rough estimate of the number of certificates
    // issued on this device. Interpreting this metric is not clearly defined and is not particularly useful for us.

    // 6. Obtain a reference to the ASN.1 parser library that is most appropriate for your toolset. Find the nearest
    //    certificate to the root that contains the key attestation certificate extension. If the provisioning
    //    information certificate extension was present, the key attestation certificate extension must be in the
    //    immediately subsequent certificate. Use the parser to extract the key attestation certificate extension data
    //    from that certificate.
    let key_attestation = x509_certificates
        .iter()
        .find_map(|cert| KeyAttestationExtension::parse_key_description(cert).transpose())
        .transpose()?
        .ok_or(GoogleKeyAttestationError::NoKeyAttestationExtension)?;

    // 7. Check the extension data that you've retrieved in the previous steps for consistency and compare with the set
    //    of values that you expect the hardware-backed key to contain.
    key_attestation.verify(attestation_challenge)?;

    Ok(x509_certificates.pop().unwrap())
}

fn verify_google_attestation_certificate_chain<'a>(
    certificate_chain: &'a [CertificateDer],
    root_public_keys: &[RootPublicKey],
    unix_time: UnixTime,
) -> Result<X509Certificate<'a>, GoogleKeyAttestationError> {
    let root_index = certificate_chain.len() - 1;

    // `unwrap` is safe because of guard that verifies the certificate chain is not empty.
    let root_certificate_der = certificate_chain.get(root_index).unwrap();
    let (_, root_certificate) =
        X509Certificate::from_der(root_certificate_der).map_err(GoogleKeyAttestationError::CertificateDecode)?;
    let root_public_key = root_certificate
        .public_key()
        .parsed()
        .map_err(GoogleKeyAttestationError::RootPublicKey)?;

    // Verify that the root public certificate is trustworthy.
    if !root_public_keys.iter().any(|public_key| root_public_key == *public_key) {
        return Err(GoogleKeyAttestationError::RootPublicKeyMismatch);
    }

    // Take the root certificate in the list as trust anchor. This is a hack which allows us to use
    // `EndEntityCert::verify_for_usage` to verify the certificate chain. This hack is safe, because we have verified
    // the public key of the root certificate to be trustworthy.
    let trust_anchor = webpki::anchor_from_trusted_cert(root_certificate_der)
        .map_err(GoogleKeyAttestationError::InvalidTrustAnchor)?;
    let trust_anchors = vec![trust_anchor];

    // EndEntityCert is the first certificate in the list.
    // `unwrap` is safe because of guard that verifies the certificate chain is not empty.
    let end_certificate = EndEntityCert::try_from(certificate_chain.first().unwrap())
        .map_err(GoogleKeyAttestationError::LeafCertificate)?;

    // Verify that each certificate signs the next certificate in the chain.
    let _verified_path = end_certificate
        .verify_for_usage(
            &[
                ECDSA_P256_SHA256,
                ECDSA_P256_SHA384,
                ECDSA_P384_SHA256,
                ECDSA_P384_SHA384,
                RSA_PKCS1_2048_8192_SHA256,
                RSA_PKCS1_2048_8192_SHA384,
                RSA_PKCS1_2048_8192_SHA512,
                RSA_PKCS1_3072_8192_SHA384,
            ],
            &trust_anchors,
            &certificate_chain[1..root_index],
            unix_time,
            KeyUsage::client_auth(),
            None,
            None,
        )
        .map_err(GoogleKeyAttestationError::InvalidCertificateChain)?;

    Ok(root_certificate)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use num_bigint::BigUint;
    use rstest::rstest;

    use crate::android_crl::AndroidCrlStatus;
    use crate::android_crl::RevocationStatusEntry;
    use crate::attestation_extension::key_description::KeyDescription;
    #[cfg(not(feature = "allow_emulator_keys"))]
    use crate::attestation_extension::key_description::SecurityLevel;
    use crate::mock::MockCaChain;

    use super::*;

    static MOCK_CA_CHAIN: LazyLock<MockCaChain> = LazyLock::new(|| MockCaChain::generate(1));
    static OTHER_MOCK_CA_CHAIN: LazyLock<MockCaChain> = LazyLock::new(|| MockCaChain::generate(1));

    fn key_description(challenge: &[u8]) -> KeyDescription {
        KeyDescription::new_valid_mock(challenge.to_vec())
    }

    #[cfg(not(feature = "allow_emulator_keys"))]
    fn key_description_insecure_attestation_security_level(challenge: &[u8]) -> KeyDescription {
        KeyDescription {
            attestation_security_level: SecurityLevel::Software,
            ..key_description(challenge)
        }
    }

    #[cfg(not(feature = "allow_emulator_keys"))]
    fn key_description_insecure_key_mint_security_level(challenge: &[u8]) -> KeyDescription {
        KeyDescription {
            key_mint_security_level: SecurityLevel::Software,
            ..key_description(challenge)
        }
    }

    fn revoked_intermediary_from(mock_ca: &MockCaChain) -> RevocationStatusList {
        // Prepare revocation list with a revoked intermediary
        let mut revocation_list = RevocationStatusList {
            entries: HashMap::new(),
        };
        // Get SerialNumber of intermediary
        let serial_number = mock_ca.last_ca_certificate.params().serial_number.as_ref().unwrap();
        // Insert revoked entry in the CRL
        revocation_list.entries.insert(
            BigUint::from_bytes_be(serial_number.as_ref()),
            RevocationStatusEntry {
                status: AndroidCrlStatus::Revoked,
                comment: None,
                expires: None,
                reason: None,
            },
        );

        revocation_list
    }

    fn perform_google_key_attestation(
        key_description: &KeyDescription,
        attestation_challenge: &[u8],
        certificate_mock_ca: &MockCaChain,
        trusted_root_mock_ca: &MockCaChain,
        revocation_list: &RevocationStatusList,
    ) -> Result<(), GoogleKeyAttestationError> {
        // Generate attested certificate chain
        let (certificates, _) = certificate_mock_ca.generate_attested_leaf_certificate(key_description);
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key
        let root_public_keys = vec![trusted_root_mock_ca.root_public_key.clone().into()];

        // Verify the attested key
        verify_google_key_attestation(
            &certificate_chain,
            &root_public_keys,
            revocation_list,
            attestation_challenge,
        )
        .map(|_| ())
    }

    #[test]
    fn test_google_key_attestation() {
        let challenge = b"challenge";
        perform_google_key_attestation(
            &key_description(challenge),
            challenge,
            &MOCK_CA_CHAIN,
            &MOCK_CA_CHAIN,
            &RevocationStatusList::default(),
        )
        .expect("verification should succeed");
    }

    #[rstest]
    #[case(
        key_description(b"challenge"),
        b"other_challenge",
        KeyAttestationVerificationError::AttestationChallenge
    )]
    #[cfg_attr(
        not(feature = "allow_emulator_keys"),
        case(
            key_description_insecure_attestation_security_level(b"challenge"),
            b"challenge",
            KeyAttestationVerificationError::AttestationSecurityLevel(SecurityLevel::Software)
        )
    )]
    #[cfg_attr(
        not(feature = "allow_emulator_keys"),
        case(
            key_description_insecure_key_mint_security_level(b"challenge"),
            b"challenge",
            KeyAttestationVerificationError::KeyMintSecurityLevel(SecurityLevel::Software)
        )
    )]
    fn test_google_key_attestation_invalid_attestation(
        #[case] key_description: KeyDescription,
        #[case] attestation_challenge: &[u8],
        #[case] expected_error: KeyAttestationVerificationError,
    ) {
        let error = perform_google_key_attestation(
            &key_description,
            attestation_challenge,
            &MOCK_CA_CHAIN,
            &MOCK_CA_CHAIN,
            &RevocationStatusList::default(),
        )
        .expect_err("should fail for attestation verification");
        assert_matches!(
            error,
            GoogleKeyAttestationError::KeyAttestationVerification(verification_error) if verification_error == expected_error
        )
    }

    #[test]
    fn test_google_key_attestation_invalid_root() {
        let challenge = b"challenge";

        // Verify the attested key
        let error = perform_google_key_attestation(
            &key_description(challenge),
            challenge,
            &MOCK_CA_CHAIN,
            &OTHER_MOCK_CA_CHAIN,
            &RevocationStatusList::default(),
        )
        .expect_err("should fail on root public key");
        assert_matches!(error, GoogleKeyAttestationError::RootPublicKeyMismatch)
    }

    #[test]
    fn test_google_key_attestation_revoked_certificate() {
        let challenge = b"challenge";

        // Verify the attested key
        let error = perform_google_key_attestation(
            &key_description(challenge),
            challenge,
            &MOCK_CA_CHAIN,
            &MOCK_CA_CHAIN,
            &revoked_intermediary_from(&MOCK_CA_CHAIN),
        )
        .expect_err("should fail on revoked certificates");
        assert_matches!(error, GoogleKeyAttestationError::RevokedCertificates(_))
    }

    #[test]
    fn test_google_key_attestation_without_certificate_extension() {
        let challenge = b"challenge";

        // Generate an unattested certificate chain
        let (certificates, _) = MOCK_CA_CHAIN.generate_leaf_certificate();
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key from the same MOCK_CA_CHAIN
        let root_public_keys = vec![MOCK_CA_CHAIN.root_public_key.clone().into()];

        // Verify the attested key
        let error = verify_google_key_attestation(
            &certificate_chain,
            &root_public_keys,
            &RevocationStatusList::default(),
            challenge,
        )
        .expect_err("should fail because certificate extension not found");
        assert_matches!(error, GoogleKeyAttestationError::NoKeyAttestationExtension);
    }

    #[test]
    fn test_google_key_attestation_invalid_certificate_chain() {
        let challenge = b"challenge";

        // Generate a certificate chain with an invalid intermediate
        let certificates = {
            let (mut certificates, _) = MOCK_CA_CHAIN.generate_attested_leaf_certificate(&key_description(challenge));
            let (mut certificates_1, _) = OTHER_MOCK_CA_CHAIN.generate_leaf_certificate();

            certificates[1] = certificates_1.remove(1);
            certificates
        };
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key from the same MOCK_CA_CHAIN
        let root_public_keys = vec![MOCK_CA_CHAIN.root_public_key.clone().into()];

        // Verify the attested key
        let error = verify_google_key_attestation(
            &certificate_chain,
            &root_public_keys,
            &RevocationStatusList::default(),
            challenge,
        )
        .expect_err("should fail because of invalid certificate chain");
        assert_matches!(error, GoogleKeyAttestationError::InvalidCertificateChain(_));
    }

    #[test]
    fn test_google_key_attestation_expired_certificate() {
        let challenge = b"challenge";

        // Generate a certificate chain with an expired intermediate
        let (certificates, _) = MOCK_CA_CHAIN.generate_expired_leaf_certificate(&key_description(challenge));
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key from the same MOCK_CA_CHAIN
        let root_public_keys = vec![MOCK_CA_CHAIN.root_public_key.clone().into()];

        // Verify the attested key
        let error = verify_google_key_attestation(
            &certificate_chain,
            &root_public_keys,
            &RevocationStatusList::default(),
            challenge,
        )
        .expect_err("should fail because of invalid certificate chain");
        assert_matches!(error, GoogleKeyAttestationError::InvalidCertificateChain(_));
    }
}
