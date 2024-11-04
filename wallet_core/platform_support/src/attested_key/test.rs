use std::{convert::Infallible, fmt::Debug, mem};

use apple_app_attest::{
    AppIdentifier, Assertion, Attestation, AttestationEnvironment, ClientData, APPLE_TRUST_ANCHORS,
};
use chrono::Utc;

use wallet_common::utils;

use super::{AppleAttestedKey, AttestedKey, AttestedKeyHolder, KeyWithAttestation};

/// This simply wraps a [`Vec<u8>`] as hash data and a generated challenge.
struct SimpleClientData {
    hash_data: Vec<u8>,
    challenge: Vec<u8>,
}

impl SimpleClientData {
    pub fn new(hash_data: Vec<u8>) -> Self {
        Self {
            hash_data,
            challenge: utils::random_bytes(32),
        }
    }
}

impl ClientData for SimpleClientData {
    type Error = Infallible;

    fn hash_data(&self) -> Result<impl AsRef<[u8]>, Self::Error> {
        Ok(&self.hash_data)
    }

    fn challenge(&self) -> impl AsRef<[u8]> {
        &self.challenge
    }
}

pub async fn create_and_verify_attested_key<H>(holder: H, challenge: Vec<u8>, payload: Vec<u8>)
where
    H: AttestedKeyHolder,
    <H as AttestedKeyHolder>::AppleKey: Debug,
    <H as AttestedKeyHolder>::GoogleKey: Debug,
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
            // When Xcode compiles the crate as part of the integration tests,
            // the environment variables below should be set.
            let (Some(team_id), Some(bundle_id)) = (
                option_env!("DEVELOPMENT_TEAM"),
                option_env!("PRODUCT_BUNDLE_IDENTIFIER"),
            ) else {
                panic!("Xcode environment variables are not defined")
            };
            let app_identifier = AppIdentifier::new(team_id, bundle_id);

            // Perform the server side check of the attestation here.
            let (_, public_key) = Attestation::parse_and_verify(
                &attestation_data,
                &APPLE_TRUST_ANCHORS,
                Utc::now(),
                &challenge,
                &app_identifier,
                AttestationEnvironment::Development, // Assume that tests use the AppAttest sandbox
            )
            .expect("could not verify attestation");

            // Create an assertion for the payload and perform the server side check,
            // using the public key extracted during attestation in the previous step.
            let client_data1 = SimpleClientData::new(payload.clone());
            let assertion1 = key.sign(payload.clone()).await.expect("could not sign payload");

            Assertion::parse_and_verify(
                assertion1.as_ref(),
                &client_data1,
                &public_key,
                &app_identifier,
                0,
                client_data1.challenge().as_ref(),
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

            let client_data2 = SimpleClientData::new(payload.clone());
            let assertion2 = key.sign(payload).await.expect("could not sign payload a second time");

            Assertion::parse_and_verify(
                assertion2.as_ref(),
                &client_data2,
                &public_key,
                &app_identifier,
                1,
                client_data2.challenge().as_ref(),
            )
            .expect("could not verify second assertion");
        }
        KeyWithAttestation::Google { .. } => panic!("Google key/app attestation is currently unimplemented"),
    };
}
