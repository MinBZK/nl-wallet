use futures::FutureExt;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use rand_core::OsRng;
use rustls_pki_types::CertificateDer;

use android_attest::android_crl::RevocationStatusList;
use android_attest::attestation_extension::key_description::KeyDescription;
use android_attest::certificate_chain::verify_google_key_attestation;
use android_attest::mock_chain::MockCaChain;
use android_attest::root_public_key::RootPublicKey;
use apple_app_attest::AppIdentifier;
use apple_app_attest::AssertionCounter;
use apple_app_attest::AttestationEnvironment;
use apple_app_attest::MockAttestationCa;
use apple_app_attest::VerifiedAttestation;
use platform_support::attested_key::mock::MockAppleAttestedKey;
use wallet_common::utils;

use wallet_account::messages::registration::Registration;
use wallet_account::messages::registration::RegistrationAttestation;
use wallet_account::signed::ChallengeResponse;
use wallet_account::signed::SequenceNumberComparison;

#[test]
fn test_apple_registration() {
    // The Wallet Provider generates a challenge.
    let challenge = b"challenge";

    // Generate a mock assertion, a mock attested key and a mock PIN siging key.
    let environment = AttestationEnvironment::Development;
    let app_identifier = AppIdentifier::new_mock();
    let mock_ca = MockAttestationCa::generate();
    let (attested_key, attestation) =
        MockAppleAttestedKey::new_with_attestation(&mock_ca, challenge, environment, app_identifier.clone());
    let pin_signing_key = SigningKey::random(&mut OsRng);

    // The Wallet generates a registration message.
    let msg =
        ChallengeResponse::<Registration>::new_apple(&attested_key, attestation, &pin_signing_key, challenge.to_vec())
            .now_or_never()
            .unwrap()
            .expect("challenge response with apple registration should be created successfully");

    let unverified = msg
        .dangerous_parse_unverified()
        .expect("registration should parse successfully");
    let RegistrationAttestation::Apple { data: attestation_data } = &unverified.payload.attestation else {
        panic!("apple registration message should contain attestation data");
    };

    let (_attestation, public_key) = VerifiedAttestation::parse_and_verify(
        attestation_data,
        &[mock_ca.trust_anchor()],
        challenge,
        &app_identifier,
        environment,
    )
    .expect("apple attestation should validate succesfully");

    // The Wallet Provider takes the public keys from the message and verifies the signatures.
    msg.parse_and_verify_apple(
        challenge,
        SequenceNumberComparison::EqualTo(0),
        &public_key,
        &app_identifier,
        AssertionCounter::default(),
        unverified.payload.pin_pubkey.as_inner(),
    )
    .expect("apple registration should verify successfully");
}

#[test]
fn test_google_registration() {
    // The Wallet Provider generates a challenge.
    let challenge = b"challenge";

    // Generate a mock certificate chain, a random app attestation token and a mock PIN signing key.
    let attested_ca_chain = MockCaChain::generate(1);
    let (attested_certificate_chain, attested_private_key) =
        attested_ca_chain.generate_attested_leaf_certificate(&KeyDescription::new_valid_mock(challenge.to_vec()));
    let integrity_token = utils::random_string(32);
    let pin_signing_key = SigningKey::random(&mut OsRng);

    // The Wallet generates a registration message.
    let msg = ChallengeResponse::<Registration>::new_google(
        &attested_private_key,
        attested_certificate_chain.try_into().unwrap(),
        integrity_token,
        &pin_signing_key,
        challenge.to_vec(),
    )
    .now_or_never()
    .unwrap()
    .expect("challenge response with google registration should be created successfully");

    let unverified = msg
        .dangerous_parse_unverified()
        .expect("registration should parse successfully");
    let RegistrationAttestation::Google { certificate_chain, .. } = &unverified.payload.attestation else {
        panic!("google registration message should contain certificate chain");
    };

    // Verify mock certificate chain and extract the leaf certificate public key.
    let der_certificate_chain = certificate_chain
        .as_ref()
        .iter()
        .map(|der| CertificateDer::from_slice(der))
        .collect::<Vec<_>>();
    let root_public_keys = vec![RootPublicKey::Rsa(attested_ca_chain.root_public_key.clone())];

    let certificate = verify_google_key_attestation(
        &der_certificate_chain,
        &root_public_keys,
        &RevocationStatusList::default(),
        challenge,
    )
    .unwrap();

    let attested_public_key = VerifyingKey::from_public_key_der(certificate.public_key().raw).unwrap();

    // The Wallet Provider takes the public keys from the message and verifies the signatures.
    msg.parse_and_verify_google(
        challenge,
        SequenceNumberComparison::EqualTo(0),
        &attested_public_key,
        unverified.payload.pin_pubkey.as_inner(),
    )
    .expect("google registration should verify successfully");
}
