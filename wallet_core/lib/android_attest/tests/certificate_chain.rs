use std::collections::HashMap;
use std::sync::LazyLock;

use android_attest::certificate_chain::verify_google_key_attestation_with_params;
use android_attest::sig_alg::ECDSA_P256_SHA256_WITH_NULL_PARAMETERS;
use assert_matches::assert_matches;
use chrono::Utc;
use num_bigint::BigUint;
use rstest::rstest;
use rustls_pki_types::CertificateDer;

use android_attest::android_crl::AndroidCrlStatus;
use android_attest::android_crl::RevocationStatusEntry;
use android_attest::android_crl::RevocationStatusList;
use android_attest::attestation_extension::key_attestation::KeyAttestationVerificationError;
use android_attest::attestation_extension::key_description::KeyDescription;
#[cfg(not(feature = "allow_emulator_keys"))]
use android_attest::attestation_extension::key_description::SecurityLevel;
use android_attest::certificate_chain::verify_google_key_attestation;
use android_attest::certificate_chain::GoogleKeyAttestationError;
use android_attest::mock_chain::MockCaChain;
use android_attest::root_public_key::RootPublicKey;
use webpki::ring::ECDSA_P256_SHA256;
use webpki::ring::RSA_PKCS1_2048_8192_SHA256;

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
    let root_public_keys = vec![RootPublicKey::Rsa(trusted_root_mock_ca.root_public_key.clone())];

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
    let certificates = MOCK_CA_CHAIN.generate_leaf_certificate();
    let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

    // Get root public key from the same MOCK_CA_CHAIN
    let root_public_keys = vec![RootPublicKey::Rsa(MOCK_CA_CHAIN.root_public_key.clone())];

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
        let mut certificates_1 = OTHER_MOCK_CA_CHAIN.generate_leaf_certificate();

        certificates[1] = certificates_1.remove(1);
        certificates
    };
    let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

    // Get root public key from the same MOCK_CA_CHAIN
    let root_public_keys = vec![RootPublicKey::Rsa(MOCK_CA_CHAIN.root_public_key.clone())];

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
    let certificates = MOCK_CA_CHAIN.generate_expired_leaf_certificate(&key_description(challenge));
    let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

    // Get root public key from the same MOCK_CA_CHAIN
    let root_public_keys = vec![RootPublicKey::Rsa(MOCK_CA_CHAIN.root_public_key.clone())];

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
fn test_google_key_attestation_signature_algorithm_with_null_parameters() {
    let attestation_challenge = b"challenge";
    let key_description = key_description(attestation_challenge);
    let certificate_mock_ca = &MOCK_CA_CHAIN;
    let trusted_root_mock_ca = &MOCK_CA_CHAIN;
    let revocation_list = &RevocationStatusList::default();

    // Generate attested certificate chain
    let certificates =
        certificate_mock_ca.generate_attested_leaf_certificate_with_null_sig_parameters(&key_description);

    let certificate_chain: Vec<_> = certificates.iter().map(|der| CertificateDer::from_slice(der)).collect();

    // Get root public key
    let root_public_keys = vec![RootPublicKey::Rsa(trusted_root_mock_ca.root_public_key.clone())];

    let supported_sig_algs = vec![
        ECDSA_P256_SHA256,
        ECDSA_P256_SHA256_WITH_NULL_PARAMETERS,
        RSA_PKCS1_2048_8192_SHA256,
    ];

    // Verify the attested key
    verify_google_key_attestation_with_params(
        &certificate_chain,
        &root_public_keys,
        revocation_list,
        attestation_challenge,
        &supported_sig_algs,
        Utc::now(),
    )
    .expect("valid certificate chain");
}
