use std::iter;
use std::rc::Rc;

use der::Any;
use der::Decode;
use der::Encode;
use der::asn1::BitString;
use der::asn1::ObjectIdentifier;
use derive_more::Debug;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::Signer;
use p256::pkcs8::DecodePrivateKey;
use rand::Rng;
use rcgen::BasicConstraints;
use rcgen::CertificateParams;
use rcgen::CustomExtension;
use rcgen::DistinguishedName;
use rcgen::DnType;
use rcgen::IsCa;
use rcgen::KeyPair;
use rcgen::PKCS_ECDSA_P256_SHA256;
use rcgen::PKCS_RSA_SHA256;
use rcgen::RsaKeySize;
use rsa::RsaPublicKey;
use spki::AlgorithmIdentifierOwned;
use spki::DecodePublicKey;
use x509_cert::certificate::Certificate;

use crate::attestation_extension::KEY_ATTESTATION_EXTENSION_OID;
use crate::attestation_extension::key_description::KeyDescription;

/// Represents a Google CA with a variable number of intermediates. After creation,
/// this can be used to generate leaf certificates to emulate Android key attestation.
// TODO: Include a mock key attestation certificate extension.
#[derive(Debug)]
pub struct MockCaChain {
    certificates_der: Vec<Vec<u8>>,
    pub root_public_key: RsaPublicKey,
    #[debug("{:?}", last_ca_certificate.der())]
    pub last_ca_certificate: rcgen::Certificate,
    last_ca_key_pair: rcgen::KeyPair,
}

impl MockCaChain {
    /// Generate a chain of CAs, with the requested number of intermediates.
    /// If no intermediates are requested, only a root CA is generated.
    pub fn generate(intermediate_count: u8) -> Self {
        let mut rng = rand::thread_rng();

        // Start with an iterator that runs backwards from `intermediate_count` down to and including 0.
        let mut certificates_and_key_pairs = (0..=intermediate_count)
            .rev()
            .scan(
                None,
                |prev_cert_and_pair: &mut Option<Rc<(rcgen::Certificate, rcgen::KeyPair)>>, constrained_count| {
                    // Generate an RSA key pair as root CA and ECDSA key pairs as intermediate CAs and set the `IsCa`
                    // value as certificate parameters, using a decrementing intermediate count as constraint.
                    // The intermediates are ECDSA keys as in 'real' certificate chains the final keys in the chain
                    // are ECDSA also we need this for `generate_attested_leaf_certificate_with_null_sig_parameters`.
                    let key_pair = if prev_cert_and_pair.is_some() {
                        KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap()
                    } else {
                        KeyPair::generate_rsa_for(&PKCS_RSA_SHA256, RsaKeySize::_2048).unwrap()
                    };

                    let mut params = CertificateParams::default();
                    params.serial_number = Some((rng.r#gen::<u64>()).into());
                    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(constrained_count));

                    // Create a SubjectDN using the constrained_count to make it uniquely recognizable.
                    let mut distinguished_name = DistinguishedName::new();
                    distinguished_name.push(DnType::CommonName, format!("cert {constrained_count}"));
                    params.distinguished_name = distinguished_name;

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
        let root_public_key = RsaPublicKey::from_public_key_der(&root_key_pair.public_key_der()).unwrap();

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

    fn generate_certificate(&self, params: CertificateParams) -> (Vec<Vec<u8>>, SigningKey) {
        // Generate a leaf certificate and convert it to DER.
        let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let certificate = params
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

    fn modify_certificate_signature_algorithm_to(
        &self,
        certificate: &[u8],
        signature_algorithm: AlgorithmIdentifierOwned,
    ) -> Vec<u8> {
        // Decode certificate
        let mut certificate = Certificate::from_der(certificate).unwrap();
        // Modify signature algorithm
        certificate.tbs_certificate.signature = signature_algorithm.clone();
        // Re-encode tbsCertificate
        let tbs_certificate_der = certificate.tbs_certificate.to_der().unwrap();

        // Obtain signing key from chain
        let signing_key = SigningKey::from_pkcs8_der(self.last_ca_key_pair.serialized_der()).unwrap();

        // Calculate signature over digest
        let signature: Signature = signing_key.sign(&tbs_certificate_der);

        // Set signature and signature algorithm on certificate
        certificate.signature = BitString::from_bytes(signature.to_der().as_bytes()).unwrap();
        certificate.signature_algorithm = signature_algorithm;

        // Encode certificate
        certificate.to_der().unwrap()
    }

    /// Generates a new leaf certificate, returning both the full certificate
    /// chain containing this leaf and the its corresponding private key.
    pub fn generate_leaf_certificate(&self) -> Vec<Vec<u8>> {
        let (chain, _) = self.generate_certificate(CertificateParams::default());
        chain
    }

    /// Generates a new leaf certificate including the android key attestation extension.
    /// Returns both the full certificate chain containing this leaf and the its corresponding private key.
    pub fn generate_attested_leaf_certificate(&self, key_description: &KeyDescription) -> (Vec<Vec<u8>>, SigningKey) {
        let mut certificate_params = CertificateParams::default();
        certificate_params
            .custom_extensions
            .push(CustomExtension::from_oid_content(
                &KEY_ATTESTATION_EXTENSION_OID.iter().unwrap().collect::<Vec<u64>>(),
                rasn::der::encode(key_description).unwrap(),
            ));

        self.generate_certificate(certificate_params)
    }

    /// Generates a new leaf certificate including the android key attestation extension.
    /// Returns both the full certificate chain containing this leaf and the its corresponding private key.
    pub fn generate_attested_leaf_certificate_with_null_sig_parameters(
        &self,
        key_description: &KeyDescription,
    ) -> Vec<Vec<u8>> {
        let mut certificate_params = CertificateParams::default();
        certificate_params
            .custom_extensions
            .push(CustomExtension::from_oid_content(
                &KEY_ATTESTATION_EXTENSION_OID.iter().unwrap().collect::<Vec<u64>>(),
                rasn::der::encode(key_description).unwrap(),
            ));

        let (mut certificate_chain, _) = self.generate_certificate(certificate_params);

        let leaf_certificate = certificate_chain.remove(0);

        let modified_certificate = self.modify_certificate_signature_algorithm_to(
            &leaf_certificate,
            AlgorithmIdentifierOwned {
                oid: ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2"),
                parameters: Some(Any::null()),
            },
        );

        certificate_chain.insert(0, modified_certificate);

        certificate_chain
    }

    /// Generates a new leaf certificate including the android key attestation extension, that expired yesterday.
    /// Returns both the full certificate chain containing this leaf and the its corresponding private key.
    pub fn generate_expired_leaf_certificate(&self, key_description: &KeyDescription) -> Vec<Vec<u8>> {
        let mut certificate_params = CertificateParams::default();
        certificate_params.not_after = time::OffsetDateTime::now_utc() - time::Duration::days(1);
        certificate_params
            .custom_extensions
            .push(CustomExtension::from_oid_content(
                &KEY_ATTESTATION_EXTENSION_OID.iter().unwrap().collect::<Vec<u64>>(),
                rasn::der::encode(key_description).unwrap(),
            ));

        let (chain, _) = self.generate_certificate(certificate_params);
        chain
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use assert_matches::assert_matches;
    use rsa::BigUint;
    use rsa::traits::PublicKeyParts;
    use rstest::rstest;
    use rustls_pki_types::CertificateDer;
    use rustls_pki_types::SignatureVerificationAlgorithm;
    use rustls_pki_types::UnixTime;
    use webpki::EndEntityCert;
    use webpki::KeyUsage;
    use webpki::ring::ECDSA_P256_SHA256;
    use webpki::ring::RSA_PKCS1_2048_8192_SHA256;
    use x509_parser::prelude::FromDer;
    use x509_parser::prelude::X509Certificate;
    use x509_parser::public_key::PublicKey;

    use crate::attestation_extension::key_description::KeyDescription;
    use crate::sig_alg::ECDSA_P256_SHA256_WITH_NULL_PARAMETERS;

    use super::MockCaChain;

    #[rstest]
    fn test_generate_mock_ca_chain(#[values(0, 1, 3, 6)] intermediate_count: u8) {
        let mock_ca_chain = MockCaChain::generate(intermediate_count);
        let certificates = mock_ca_chain.generate_leaf_certificate();

        assert_eq!(certificates.len(), usize::from(intermediate_count) + 2);

        let supported_sig_algs = [RSA_PKCS1_2048_8192_SHA256, ECDSA_P256_SHA256];

        verify_certificate_chain(&certificates, &mock_ca_chain, intermediate_count, &supported_sig_algs)
            .expect("certificate chain should verify");
    }

    #[test]
    fn test_generate_mock_ca_chain_and_leaf_with_ecdsa_null_sig_parameters() {
        let intermediate_count = 1;
        let mock_ca_chain = MockCaChain::generate(intermediate_count);
        let certificates = mock_ca_chain.generate_attested_leaf_certificate_with_null_sig_parameters(
            &KeyDescription::new_valid_mock(b"challenge".to_vec()),
        );

        assert_eq!(certificates.len(), usize::from(intermediate_count) + 2);

        let supported_sig_algs = [RSA_PKCS1_2048_8192_SHA256, ECDSA_P256_SHA256];
        let error = verify_certificate_chain(&certificates, &mock_ca_chain, intermediate_count, &supported_sig_algs)
            .expect_err("certificate chain should not verify");

        assert_matches!(error, webpki::Error::UnsupportedSignatureAlgorithm);

        let supported_sig_algs = [
            RSA_PKCS1_2048_8192_SHA256,
            ECDSA_P256_SHA256,
            ECDSA_P256_SHA256_WITH_NULL_PARAMETERS,
        ];
        verify_certificate_chain(&certificates, &mock_ca_chain, intermediate_count, &supported_sig_algs)
            .expect("certificate chain should verify");
    }

    fn verify_certificate_chain<'a>(
        certificates: &'a [Vec<u8>],
        mock_ca_chain: &'a MockCaChain,
        intermediate_count: u8,
        supported_sig_algs: &[&dyn SignatureVerificationAlgorithm],
    ) -> Result<(), webpki::Error> {
        let root_certificate_der = CertificateDer::from_slice(certificates.last().unwrap());
        let (_, root_certificate) = X509Certificate::from_der(&root_certificate_der).unwrap();
        let root_public_key = root_certificate.public_key().parsed().unwrap();

        if let PublicKey::RSA(rsa_public_key) = root_public_key {
            assert_eq!(
                BigUint::from_bytes_be(rsa_public_key.modulus),
                *mock_ca_chain.root_public_key.n()
            );
            assert_eq!(
                BigUint::from_bytes_be(rsa_public_key.exponent),
                *mock_ca_chain.root_public_key.e()
            );
        } else {
            panic!("root public key should be RSA public key");
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
                supported_sig_algs,
                &[trust_anchor],
                &intermediate_certs,
                UnixTime::since_unix_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap()),
                KeyUsage::client_auth(),
                None,
                None,
            )
            .map(|_| ())
    }
}
