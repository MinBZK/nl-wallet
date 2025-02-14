use std::sync::Arc;

use async_dropper::AsyncDropper;
use futures::Future;
use hsm::service::HsmError;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use rstest::rstest;
use rstest::Context;
use serial_test::serial;

use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Decrypter;
use hsm::model::encrypter::Encrypter;
use hsm::model::mock::MockPkcs11Client;
use hsm::model::Hsm;
use hsm::service::Pkcs11Client;
use hsm::service::Pkcs11Hsm;
use hsm::test::TestCase;
use wallet_common::utils::random_bytes;

#[rstest]
#[case::sign_sha256_hmac(sign_sha256_hmac)]
#[case::sign_ecdsa(sign_ecdsa)]
#[case::encrypt_decrypt(encrypt_decrypt)]
#[case::encrypt_decrypt_verifying_key(encrypt_decrypt_verifying_key)]
#[case::wrap_key_and_sign(wrap_key_and_sign)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn hsm_tests<F, Fut>(#[context] ctx: Context, #[case] test: F)
where
    F: FnOnce(TestCase<Pkcs11Hsm>) -> Fut,
    Fut: Future<Output = TestCase<Pkcs11Hsm>>,
{
    let test_case = TestCase::new("hsm.toml", ctx.description.unwrap());
    let test_case = test(test_case).await;
    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[rstest]
#[case::sign_sha256_hmac(sign_sha256_hmac)]
#[case::sign_ecdsa(sign_ecdsa)]
#[case::encrypt_decrypt(encrypt_decrypt)]
#[case::encrypt_decrypt_verifying_key(encrypt_decrypt_verifying_key)]
// #[case::wrap_key_and_sign(wrap_key_and_sign)] // TODO: generate aes key is unsupported for Mock
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn hsm_mock_tests<F, Fut>(#[context] ctx: Context, #[case] test: F)
where
    F: FnOnce(TestCase<MockPkcs11Client<HsmError>>) -> Fut,
    Fut: Future<Output = TestCase<MockPkcs11Client<HsmError>>>,
{
    let test_case = TestCase::mock(ctx.description.unwrap());
    test(test_case).await;
}

async fn sign_sha256_hmac<H>(test_case: TestCase<H>) -> TestCase<H>
where
    H: Hsm,
{
    let (hsm, identifier) = test_case.test_params();
    let data = Arc::new(random_bytes(32));

    Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();
    let signature = hsm.sign_hmac(identifier, Arc::clone(&data)).await.unwrap();
    hsm.verify_hmac(identifier, Arc::clone(&data), signature).await.unwrap();

    test_case
}

async fn sign_ecdsa<H>(test_case: TestCase<H>) -> TestCase<H>
where
    H: Hsm,
{
    let (hsm, identifier) = test_case.test_params();
    let data = Arc::new(random_bytes(32));

    Hsm::generate_signing_key_pair(hsm, identifier).await.unwrap();

    let signature = hsm.sign_ecdsa(identifier, Arc::clone(&data)).await.unwrap();
    let verifying_key = Hsm::get_verifying_key(hsm, identifier).await.unwrap();
    verifying_key.verify(&data, &signature).unwrap();

    test_case
}

async fn encrypt_decrypt<H>(test_case: TestCase<H>) -> TestCase<H>
where
    H: Hsm,
{
    let (hsm, identifier) = test_case.test_params();
    let data = random_bytes(32);

    Hsm::generate_aes_encryption_key(hsm, identifier).await.unwrap();

    let encrypted: Encrypted<Vec<u8>> = Hsm::encrypt(hsm, identifier, data.clone()).await.unwrap();
    assert_ne!(data.clone(), encrypted.data.clone());

    let decrypted = Hsm::decrypt(hsm, identifier, encrypted).await.unwrap();
    assert_eq!(data, decrypted);

    test_case
}

async fn encrypt_decrypt_verifying_key<H>(test_case: TestCase<H>) -> TestCase<H>
where
    H: Hsm + Encrypter<VerifyingKey> + Decrypter<VerifyingKey>,
{
    let (hsm, identifier) = test_case.test_params();

    Hsm::generate_aes_encryption_key(hsm, identifier).await.unwrap();

    let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
    let encrypted = Encrypter::encrypt(hsm, identifier, verifying_key).await.unwrap();

    let decrypted = Decrypter::decrypt(hsm, identifier, encrypted).await.unwrap();

    assert_eq!(verifying_key, decrypted);

    test_case
}

async fn wrap_key_and_sign<H>(test_case: TestCase<H>) -> TestCase<H>
where
    H: Pkcs11Client,
{
    let (hsm, identifier) = test_case.test_params();

    let _ = Pkcs11Client::generate_aes_encryption_key(hsm, identifier)
        .await
        .unwrap();

    let (public_key, wrapped) = hsm.generate_wrapped_key(identifier).await.unwrap();

    assert_eq!(public_key, *wrapped.public_key());

    let data = Arc::new(random_bytes(32));
    let signature = Pkcs11Client::sign_wrapped(hsm, identifier, wrapped, Arc::clone(&data))
        .await
        .unwrap();

    public_key.verify(data.as_ref(), &signature).unwrap();

    test_case
}
