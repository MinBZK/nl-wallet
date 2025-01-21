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
use crate::attestation_extension::key_attestation::KeyAttestation;
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
    #[error("could not decode root certificate from chain: {0}")]
    RootCertificateDecode(#[source] x509_parser::nom::Err<X509Error>),
    #[error("could not decode end entity certificate from leaf certificate: {0}")]
    LeafCertificate(#[source] webpki::Error),
    #[error("could not decode public key from root certificate: {0}")]
    RootPublicKey(#[source] X509Error),
    #[error("root CA in certificate chain does not contain any of the configured public keys")]
    RootPublicKeyMismatch,
    #[error("certificate chain contains revoked certificates")]
    RevokedCertificates,
    #[error("no key attestation extension found in certificate chain")]
    NoKeyAttestationExtension,
    #[error("could not extract key attestation extension: {0}")]
    KeyAttestationExtension(#[from] KeyAttestationExtensionError),
    #[error("key attestation extension is not strong enough: {0}")]
    KeyAttestationVerification(#[from] KeyAttestationVerificationError),
}

// This function implements the steps as described in: [Verify hardware-backed key pairs with key attestation](https://developer.android.com/privacy-and-security/security-key-attestation).
// The first steps in the procedure are executed on the Android device, and are the prerequisite for this function.
//
// 1. Use a KeyStore object's getCertificateChain() method to get a reference to the chain of X.509 certificates
//    associated with the hardware-backed keystore.
//
// 2. Send the certificates to a separate server that you trust for validation.
pub fn verify_google_key_attestation(
    certificate_chain: &[CertificateDer],
    root_public_keys: &[RootPublicKey],
    revocation_list: &RevocationStatusList,
) -> Result<(), GoogleKeyAttestationError> {
    assert!(!certificate_chain.is_empty());

    // 3. Obtain a reference to the X.509 certificate chain parsing and validation library that is most appropriate for
    //    your toolset. Verify that the root public certificate is trustworthy and that each certificate signs the next
    //    certificate in the chain.
    verify_google_attestation_certificate_chain(certificate_chain, root_public_keys)?;

    // 4. Check each certificate's revocation status to ensure that none of the certificates have been revoked.
    let x509_certificates = certificate_chain
        .iter()
        .map(|der| X509Certificate::from_der(der.as_ref()).map(|(_, cert)| cert))
        .collect::<Result<Vec<_>, _>>()
        .map_err(GoogleKeyAttestationError::CertificateDecode)?;

    let revoked_certificates = revocation_list.get_revoked_certificates(&x509_certificates);
    if !revoked_certificates.is_empty() {
        let revocation_log = revoked_certificates
            .iter()
            .map(|(cert, reason)| {
                format!(
                    "subject: {}, serial: {}, status: {:?}",
                    cert.subject,
                    cert.raw_serial_as_string(),
                    reason
                )
            })
            .collect::<Vec<_>>();
        tracing::error!("revoked certificates in certificate chain: {:?}", revocation_log);
        return Err(GoogleKeyAttestationError::RevokedCertificates);
    }

    // 5. Optionally, inspect the provisioning information certificate extension that is only present in newer certificate chains.
    //    We skip this step, as interpreting the provisioning information is not clearly defined.

    // 6. Obtain a reference to the ASN.1 parser library that is most appropriate for your toolset. Find the nearest
    //    certificate to the root that contains the key attestation certificate extension. If the provisioning
    //    information certificate extension was present, the key attestation certificate extension must be in the
    //    immediately subsequent certificate. Use the parser to extract the key attestation certificate extension data
    //    from that certificate.
    let key_attestation_extensions = x509_certificates
        .iter()
        .rev()
        .flat_map(|cert| KeyAttestationExtension::parse_key_description(cert).transpose())
        .collect::<Result<Vec<KeyAttestation>, _>>()?;

    let key_attestation = match key_attestation_extensions.first() {
        None => return Err(GoogleKeyAttestationError::NoKeyAttestationExtension)?,
        Some(extension) => extension,
    };

    // 7. Check the extension data that you've retrieved in the previous steps for consistency and compare with the set
    //    of values that you expect the hardware-backed key to contain.
    key_attestation.verify()?;

    Ok(())
}

fn verify_google_attestation_certificate_chain(
    certificate_chain: &[CertificateDer],
    root_public_keys: &[RootPublicKey],
) -> Result<(), GoogleKeyAttestationError> {
    assert!(!certificate_chain.is_empty());

    let root_id = certificate_chain.len() - 1;

    // `unwrap` is safe because of guard that verifies the certificate chain is not empty.
    let root_certificate_der = certificate_chain.get(root_id).unwrap();
    let (_, root_certificate) =
        X509Certificate::from_der(root_certificate_der).map_err(GoogleKeyAttestationError::RootCertificateDecode)?;
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
    let end_certificate: EndEntityCert = EndEntityCert::try_from(certificate_chain.first().unwrap())
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
            &certificate_chain[1..root_id],
            UnixTime::now(),
            KeyUsage::client_auth(),
            None,
            None,
        )
        .map_err(GoogleKeyAttestationError::InvalidCertificateChain)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rasn::types::OctetString;

    use crate::{
        attestation_extension::key_description::{KeyDescription, SecurityLevel},
        mock::MockCaChain,
    };

    use super::*;

    #[test]
    fn test_google_key_attestation() {
        let key_description = KeyDescription {
            attestation_version: 200.into(),
            attestation_security_level: SecurityLevel::TrustedEnvironment,
            key_mint_version: 300.into(),
            key_mint_security_level: SecurityLevel::TrustedEnvironment,
            attestation_challenge: OctetString::copy_from_slice(b"challenge"),
            unique_id: OctetString::copy_from_slice(b"unique_id"),
            software_enforced: Default::default(),
            hardware_enforced: Default::default(),
        };

        // Generate root and intermediate ca.
        let mock_ca_chain = MockCaChain::generate(1);
        let (certificates, _signing_keys) = mock_ca_chain.generate_attested_leaf_certificate(&key_description);
        let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

        // Get root public key, the chain length is 3 now, root + intermediate + leaf.
        let (_, root_certificate) = X509Certificate::from_der(&certificates[2]).unwrap();
        let root_public_keys = vec![RootPublicKey::rsa_from_der(root_certificate.public_key().raw).unwrap()];

        let revocation_list = RevocationStatusList::default();

        verify_google_key_attestation(&certificate_chain, &root_public_keys, &revocation_list).unwrap();
    }
}
