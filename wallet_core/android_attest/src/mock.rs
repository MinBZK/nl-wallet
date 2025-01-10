use std::iter;
use std::rc::Rc;

use derive_more::Debug;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::DecodePublicKey;
use rcgen::BasicConstraints;
use rcgen::CertificateParams;
use rcgen::IsCa;
use rcgen::KeyPair;
use rcgen::PKCS_ECDSA_P256_SHA256;

/// Represents a Google CA with a variable number of intermediates. After creation,
/// this can be used to generate leaf certificates to emulate Android key attestation.
// TODO: Include a mock key attestation certificate extension.
#[derive(Debug)]
pub struct MockCaChain {
    certificates_der: Vec<Vec<u8>>,
    pub root_public_key: VerifyingKey,
    #[debug("{:?}", last_ca_certificate.der())]
    last_ca_certificate: rcgen::Certificate,
    last_ca_key_pair: rcgen::KeyPair,
}

impl MockCaChain {
    /// Generate a chain of CAs, with the requested number of intermediates.
    /// If no intermediates are requested, only a root CA is generated.
    pub fn generate(intermediate_count: u8) -> Self {
        // Start with an iterator that runs backwards from `intermediate_count` down to and including 0.
        let mut certificates_and_key_pairs = (0..=intermediate_count)
            .rev()
            .scan(
                None,
                |prev_cert_and_pair: &mut Option<Rc<(rcgen::Certificate, rcgen::KeyPair)>>, constrained_count| {
                    // Generate a key pair and set the `IsCa` value as certificate parameters,
                    // using a decrementing intermediate count as constraint.
                    let key_pair = rcgen::KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
                    let mut params = CertificateParams::default();
                    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(constrained_count));

                    // Get the parent certificate and key pair from the previous iteration and use it to sign
                    // a new certificate. If these are not present, this is the CA, which is self-signed.
                    let certificate =
                        if let Some((parent_certificate, parent_key_pair)) = prev_cert_and_pair.as_deref() {
                            params.signed_by(&key_pair, parent_certificate, parent_key_pair)
                        } else {
                            params.self_signed(&key_pair)
                        }
                        .unwrap();

                    // Save the certificate and key pair for the next iteration,
                    // using `Rc` to keep the borrow checker happy.
                    let cert_and_pair = Rc::new((certificate, key_pair));
                    prev_cert_and_pair.replace(Rc::clone(&cert_and_pair));

                    // Return the tuple of generated certificate and key pair.
                    Some(cert_and_pair)
                },
            )
            .collect::<Vec<_>>();

        // Convert all of the X.509 certificates to DER and reverse the order,
        // so that the chain runs from leaf to root.
        let certificates_der = certificates_and_key_pairs
            .iter()
            .rev()
            .map(Rc::as_ref)
            .map(|(certificate, _)| certificate.der().to_vec())
            .collect();

        // Extract and decode the public key of the root CA.
        let (_, root_key_pair) = certificates_and_key_pairs.first().unwrap().as_ref();
        let root_public_key = VerifyingKey::from_public_key_der(&root_key_pair.public_key_der()).unwrap();

        // Save the generated certificate and key pair of the lowest level CA (which may be the root or an
        // intermediate), so that we can generate leaf certificates. As there should be only one reference
        // at this point, we can get rid of the `Rc`.
        let (last_ca_certificate, last_ca_key_pair) =
            Rc::into_inner(certificates_and_key_pairs.pop().unwrap()).unwrap();

        Self {
            certificates_der,
            root_public_key,
            last_ca_certificate,
            last_ca_key_pair,
        }
    }

    /// Generates a new leaf certificate, returning both the full certificate
    /// chain containing this leaf and the its corresponding private key.
    pub fn generate_leaf_certificate(&self) -> (Vec<Vec<u8>>, SigningKey) {
        // Generate a leaf certificate and convert it to DER.
        let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let certificate = CertificateParams::default()
            .signed_by(&key_pair, &self.last_ca_certificate, &self.last_ca_key_pair)
            .unwrap()
            .der()
            .to_vec();

        // Return a copy of the full certificate chain, including the new
        // leaf certificate and the signing key for that leaf certificate.
        let certificate_chain = iter::once(certificate)
            .chain(self.certificates_der.iter().cloned())
            .collect();
        let signing_key = SigningKey::from_pkcs8_der(key_pair.serialized_der()).unwrap();

        (certificate_chain, signing_key)
    }
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
    use x509_parser::prelude::FromDer;
    use x509_parser::prelude::X509Certificate;
    use x509_parser::public_key::PublicKey;

    use super::MockCaChain;

    #[rstest]
    fn test_generate_mock_ca_chain(#[values(0, 1, 3, 6)] intermediate_count: u8) {
        let mock_ca_chain = MockCaChain::generate(intermediate_count);
        let (certificates, _signing_key) = mock_ca_chain.generate_leaf_certificate();

        assert_eq!(certificates.len(), usize::from(intermediate_count) + 2);

        let root_certificate_der = CertificateDer::from_slice(certificates.last().unwrap());
        let (_, root_certificate) = X509Certificate::from_der(&root_certificate_der).unwrap();
        let root_public_key = root_certificate.public_key().parsed().unwrap();

        if let PublicKey::EC(ec_point) = root_public_key {
            assert_eq!(
                ec_point.data(),
                mock_ca_chain.root_public_key.to_encoded_point(false).as_bytes()
            );
        } else {
            panic!("public key shold be ECDSA verifying key");
        }

        let end_cert_der = CertificateDer::from_slice(certificates.first().unwrap());
        let end_cert =
            EndEntityCert::try_from(&end_cert_der).expect("leaf certificate should be valid end entity certificate");
        let trust_anchor = webpki::anchor_from_trusted_cert(&root_certificate_der)
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
