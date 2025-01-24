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
    use assert_matches::assert_matches;
    use rasn::types::OctetString;
    use rstest::rstest;

    use crate::attestation_extension::key_description::KeyDescription;
    use crate::attestation_extension::key_description::SecurityLevel;
    use crate::mock::MockCaChain;

    use super::*;

    fn key_description() -> KeyDescription {
        KeyDescription {
            attestation_version: 200.into(),
            attestation_security_level: SecurityLevel::TrustedEnvironment,
            key_mint_version: 300.into(),
            key_mint_security_level: SecurityLevel::TrustedEnvironment,
            attestation_challenge: OctetString::copy_from_slice(b"challenge"),
            unique_id: OctetString::copy_from_slice(b"unique_id"),
            software_enforced: Default::default(),
            hardware_enforced: Default::default(),
        }
    }

    #[cfg(not(feature = "emulator"))]
    fn key_description_insecure_attestation_security_level() -> KeyDescription {
        KeyDescription {
            attestation_security_level: SecurityLevel::Software,
            ..key_description()
        }
    }

    #[cfg(not(feature = "emulator"))]
    fn key_description_insecure_key_mint_security_level() -> KeyDescription {
        KeyDescription {
            key_mint_security_level: SecurityLevel::Software,
            ..key_description()
        }
    }

    #[test]
    fn test_google_key_attestation() {
        let key_description = key_description();

        // Generate root and intermediate ca.
        let mock_ca_chain = MockCaChain::generate(1);
        let (certificates, _signing_keys) = mock_ca_chain.generate_attested_leaf_certificate(&key_description);
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key, the chain length is 3 now, root + intermediate + leaf.
        let (_, root_certificate) = X509Certificate::from_der(&certificates[2]).unwrap();
        let root_public_keys = vec![RootPublicKey::try_from(root_certificate.public_key().raw).unwrap()];

        let revocation_list = RevocationStatusList::default();

        verify_google_key_attestation(&certificate_chain, &root_public_keys, &revocation_list, b"challenge")
            .expect("verification should succeed");
    }

    #[rstest]
    #[case(
        key_description(),
        b"other_challenge",
        KeyAttestationVerificationError::AttestationChallenge
    )]
    #[cfg_attr(
        not(feature = "emulator"),
        case(
            key_description_insecure_attestation_security_level(),
            b"challenge",
            KeyAttestationVerificationError::AttestationSecurityLevel(SecurityLevel::Software)
        )
    )]
    #[cfg_attr(
        not(feature = "emulator"),
        case(
            key_description_insecure_key_mint_security_level(),
            b"challenge",
            KeyAttestationVerificationError::KeyMintSecurityLevel(SecurityLevel::Software)
        )
    )]
    fn test_google_key_attestation_invalid_challenge(
        #[case] key_description: KeyDescription,
        #[case] attestation_challenge: &[u8],
        #[case] expected_error: KeyAttestationVerificationError,
    ) {
        // Generate root and intermediate ca.
        let mock_ca_chain = MockCaChain::generate(1);
        let (certificates, _signing_keys) = mock_ca_chain.generate_attested_leaf_certificate(&key_description);
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key, the chain length is 3 now, root + intermediate + leaf.
        let (_, root_certificate) = X509Certificate::from_der(&certificates[2]).unwrap();
        let root_public_keys = vec![RootPublicKey::try_from(root_certificate.public_key().raw).unwrap()];

        let revocation_list = RevocationStatusList::default();

        let error = verify_google_key_attestation(
            &certificate_chain,
            &root_public_keys,
            &revocation_list,
            attestation_challenge,
        )
        .expect_err("should fail for attestation verification");
        assert_matches!(
            error,
            GoogleKeyAttestationError::KeyAttestationVerification(verification_error) if verification_error == expected_error
        )
    }
}
