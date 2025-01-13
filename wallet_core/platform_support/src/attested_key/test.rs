use std::convert::Infallible;
use std::fmt::Debug;
use std::mem;

use p256::ecdsa::signature::Verifier;
use rustls_pki_types::TrustAnchor;

use apple_app_attest::AppIdentifier;
use apple_app_attest::AssertionCounter;
use apple_app_attest::AttestationEnvironment;
use apple_app_attest::ClientData;
use apple_app_attest::VerifiedAssertion;
use apple_app_attest::VerifiedAttestation;
use wallet_common::keys::EcdsaKey;
use wallet_common::utils;

use super::AppleAttestedKey;
use super::AttestedKey;
use super::AttestedKeyHolder;
use super::KeyWithAttestation;

/// This simply wraps a byte slice as hash data and a randomly generated challenge.
struct SimpleClientData<'a> {
    hash_data: &'a [u8],
    challenge: Vec<u8>,
}

impl<'a> SimpleClientData<'a> {
    pub fn new(hash_data: &'a [u8]) -> Self {
        Self {
            hash_data,
            challenge: utils::random_bytes(32),
        }
    }
}

impl ClientData for SimpleClientData<'_> {
    type Error = Infallible;

    fn hash_data(&self) -> Result<impl AsRef<[u8]>, Self::Error> {
        Ok(self.hash_data)
    }

    fn challenge(&self) -> Result<impl AsRef<[u8]>, Self::Error> {
        Ok(&self.challenge)
    }
}

pub struct AppleTestData<'a> {
    pub app_identifier: &'a AppIdentifier,
    pub trust_anchors: &'a [TrustAnchor<'a>],
}

pub async fn create_and_verify_attested_key<'a, H>(
    holder: &'a H,
    apple_test_data: Option<AppleTestData<'a>>,
    challenge: Vec<u8>,
    payload: Vec<u8>,
) where
    H: AttestedKeyHolder,
    H::AppleKey: Debug,
    H::GoogleKey: Debug,
{
    // Generate an identifier for the attested key, which on iOS also actually generates a key pair.
    let identifier = holder
        .generate()
        .await
        .expect("could not generate attested key identifier");

    // Perform key / app attestation. Note that this requires a network connection.
    let key_with_attestation = holder
        .attest(identifier.clone(), challenge.clone())
        .await
        .expect("could not perform key/app attestation");

    match key_with_attestation {
        KeyWithAttestation::Apple { key, attestation_data } => {
            let Some(apple_test_data) = apple_test_data else {
                panic!("apple test data should be provided to test");
            };

            // Perform the server side check of the attestation here.
            let (_, public_key) = VerifiedAttestation::parse_and_verify(
                &attestation_data,
                apple_test_data.trust_anchors,
                &challenge,
                apple_test_data.app_identifier,
                AttestationEnvironment::Development, // Assume that tests use the AppAttest sandbox
            )
            .expect("could not verify attestation");

            // Create an assertion for the payload and perform the server side check,
            // using the public key extracted during attestation in the previous step.
            let client_data1 = SimpleClientData::new(&payload);
            let assertion1 = key.sign(payload.clone()).await.expect("could not sign payload");

            VerifiedAssertion::parse_and_verify(
                assertion1.as_ref(),
                &client_data1,
                &public_key,
                apple_test_data.app_identifier,
                AssertionCounter::default(),
                &client_data1.challenge,
            )
            .expect("could not verify first assertion");

            // Check that we cannot have a second reference to the key while the first one is still alive.
            holder
                .attested_key(identifier.clone())
                .expect_err("creating a second attested key with the same identifier should result in an error");

            mem::drop(key);

            // Create a reference to the attested key using the identifier,
            // then generate and check another attestation.
            let AttestedKey::Apple(key) = holder
                .attested_key(identifier)
                .expect("could not instantiate attested key by identifier")
            else {
                panic!("expected Apple attested key");
            };

            let client_data2 = SimpleClientData::new(&payload);
            let assertion2 = key
                .sign(payload.clone())
                .await
                .expect("could not sign payload a second time");

            VerifiedAssertion::parse_and_verify(
                assertion2.as_ref(),
                &client_data2,
                &public_key,
                apple_test_data.app_identifier,
                AssertionCounter::from(1),
                &client_data2.challenge,
            )
            .expect("could not verify second assertion");
        }
        KeyWithAttestation::Google {
            key,
            certificate_chain: _certificate_chain,
            app_attestation_token: _app_attestation_token,
        } => {
            log::info!("Found Google Key: {key:?}");

            log::info!("Sign payload with google key");
            let signature1 = key.try_sign(&payload).await.expect("could not sign payload");

            log::info!("Obtain verifying key");
            let verifying_key1 = key.verifying_key().await.expect("could not get verifying key");

            log::info!("Verify signature with verifying key");
            verifying_key1.verify(&payload, &signature1).expect("could not verify");

            log::info!("Check we cannot recreate an attested_key with the same identifier");

            // Check that we cannot have a second reference to the key while the first one is still alive.
            holder
                .attested_key(identifier.clone())
                .expect_err("creating a second attested key with the same identifier should result in an error");

            mem::drop(key);

            log::info!("key dropped");

            // Create a reference to the attested key using the identifier,
            // then generate and check another attestation.
            let AttestedKey::Google(key) = holder
                .attested_key(identifier)
                .expect("could not instantiate attested key by identifier")
            else {
                panic!("expected Google attested key");
            };

            log::info!("Sign payload with google key");
            let signature2 = key
                .try_sign(&payload)
                .await
                .expect("could not sign payload a second time");

            log::info!("Obtain verifying key");
            let verifying_key2 = key.verifying_key().await.expect("could not get verifying key");

            assert_eq!(verifying_key1, verifying_key2);

            log::info!("Verify signature with verifying key");
            verifying_key2.verify(&payload, &signature2).expect("could not verify");
        }
    };
}
