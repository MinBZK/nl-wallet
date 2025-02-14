use std::sync::Arc;

use async_dropper::AsyncDropper;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use serial_test::serial;

use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Decrypter;
use hsm::model::encrypter::Encrypter;
use hsm::model::Hsm;
use hsm::service::Pkcs11Client;
use hsm::test::TestCase;
use wallet_common::utils::random_bytes;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn sign_sha256_hmac_using_new_secret_key() {
    let test_case = TestCase::new("hsm.toml", "sign_sha256_hmac_using_new_secret_key");
    let (hsm, identifier) = test_case.test_params();

    let data = Arc::new(random_bytes(32));

    Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();

    let signature = hsm.sign_hmac(identifier, Arc::clone(&data)).await.unwrap();

    hsm.verify_hmac(identifier, Arc::clone(&data), signature).await.unwrap();

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn sign_sha256_hmac() {
    let test_case = TestCase::new("hsm.toml", "sign_sha256_hmac");
    let (hsm, identifier) = test_case.test_params();

    let data = Arc::new(random_bytes(32));

    Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();

    let signature = hsm.sign_hmac(identifier, Arc::clone(&data)).await.unwrap();

    hsm.verify_hmac(identifier, Arc::clone(&data), signature).await.unwrap();

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn encrypt_decrypt() {
    let test_case = TestCase::new("hsm.toml", "encrypt_decrypt");
    let (hsm, identifier) = test_case.test_params();

    let data = random_bytes(32);

    hsm.generate_aes_encryption_key(identifier).await.unwrap();

    let encrypted: Encrypted<Vec<u8>> = Hsm::encrypt(hsm, identifier, data.clone()).await.unwrap();

    assert_ne!(data.clone(), encrypted.data.clone());

    let decrypted = Hsm::decrypt(hsm, identifier, encrypted).await.unwrap();

    assert_eq!(data, decrypted);

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn encrypt_decrypt_verifying_key() {
    let test_case = TestCase::new("hsm.toml", "encrypt_decrypt_verifying_key");
    let (hsm, identifier) = test_case.test_params();

    hsm.generate_aes_encryption_key(identifier).await.unwrap();

    let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
    let encrypted = Encrypter::encrypt(hsm, identifier, verifying_key).await.unwrap();

    let decrypted = Decrypter::decrypt(hsm, identifier, encrypted).await.unwrap();

    assert_eq!(verifying_key, decrypted);

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn wrap_key_and_sign() {
    let test_case = TestCase::new("hsm.toml", "generate_key_and_sign");
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

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}
