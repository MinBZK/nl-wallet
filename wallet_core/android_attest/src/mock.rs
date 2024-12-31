use std::iter;

use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use rcgen::BasicConstraints;
use rcgen::CertificateParams;
use rcgen::IsCa;
use rcgen::KeyPair;
use rcgen::PKCS_ECDSA_P256_SHA256;

/// Based on a variable number of intermediates, this generates:
/// * A chain of DER encoded X.509 certificates, which consists of a leaf, `intermediate_count` intermediates and a CA.
/// * The signing key for each certificate respectively.
// TODO: Include a mock key attestation certificate extension.
pub fn generate_mock_certificate_chain(intermediate_count: u8) -> (Vec<Vec<u8>>, Vec<SigningKey>) {
    // Start with an iterator that runs from an `IsCa` type that is constrained
    // to `intermediate_count` levels to a certificate that is not a CA.
    (0..=intermediate_count)
        .rev()
        .map(|count| IsCa::Ca(BasicConstraints::Constrained(count)))
        .chain(iter::once(IsCa::NoCa))
        .scan(None, |prev_cert_and_pair, is_ca| {
            // Generate a key pair and set the `IsCa` value as certificate parameters.
            let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
            let mut params = CertificateParams::default();
            params.is_ca = is_ca;

            // Get the parent certificate and key pair from the previous iteration and use it to sign
            // a new certificate. If these are not present, this is the CA, which is self-signed.
            let certificate = if let Some((parent_certificate, parent_key_pair)) = prev_cert_and_pair {
                params.signed_by(&key_pair, parent_certificate, parent_key_pair)
            } else {
                params.self_signed(&key_pair)
            }
            .unwrap();

            // Encode the certificate in DER and the private key in PKCS #8.
            let certificate_der = certificate.der().to_vec();
            let signing_key = SigningKey::from_pkcs8_der(key_pair.serialized_der()).unwrap();

            // Save the certificate and key pair for the next iteration.
            prev_cert_and_pair.replace((certificate, key_pair));

            // Return a tuple of the DER certificate and PKCS #8 private key.
            Some((certificate_der, signing_key))
        })
        // Collect the iterator in an intermediate `Vec` so that we can reverse it.
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        // Return a tuple of two separate `Vec`s.
        .unzip()
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use rstest::rstest;
    use rustls_pki_types::{CertificateDer, UnixTime};
    use webpki::ring::ECDSA_P256_SHA256;
    use webpki::EndEntityCert;
    use webpki::KeyUsage;

    use crate::mock::generate_mock_certificate_chain;

    #[rstest]
    fn test_generate_mock_certificate_chain(#[values(0, 1, 3, 6)] intermediate_count: u8) {
        let (certificates, signing_keys) = generate_mock_certificate_chain(intermediate_count);

        assert_eq!(certificates.len(), usize::from(intermediate_count) + 2);
        assert_eq!(signing_keys.len(), usize::from(intermediate_count) + 2);

        let end_cert_der = CertificateDer::from_slice(certificates.first().unwrap());
        let end_cert =
            EndEntityCert::try_from(&end_cert_der).expect("leaf certificate should be valid end entity certificate");
        let trust_anchor_der = CertificateDer::from_slice(certificates.last().unwrap());
        let trust_anchor = webpki::anchor_from_trusted_cert(&trust_anchor_der)
            .expect("root certificate should be a valid trust anchor");
        let intermediate_certs = certificates
            .iter()
            .skip(1)
            .take(usize::from(intermediate_count))
            .map(|cert| CertificateDer::from_slice(cert))
            .collect::<Vec<_>>();

        // Note that `webpki` seems to support a maximum of 6 intermediates.
        end_cert
            .verify_for_usage(
                &[ECDSA_P256_SHA256],
                &[trust_anchor],
                &intermediate_certs,
                UnixTime::since_unix_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap()),
                KeyUsage::client_auth(),
                None,
                None,
            )
            .expect("certificate chain should verify");
    }
}
