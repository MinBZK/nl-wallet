use async_dropper::AsyncDropper;
use rstest::Context;
use rstest::rstest;
use serial_test::serial;

use hsm::service::Pkcs11Hsm;
use hsm::test::TestCase;

#[rstest]
#[case::sign_sha256_hmac(TestCase::sign_sha256_hmac)]
#[case::sign_ecdsa(TestCase::sign_ecdsa)]
#[case::encrypt_decrypt(TestCase::encrypt_decrypt)]
#[case::encrypt_decrypt_verifying_key(TestCase::encrypt_decrypt_verifying_key)]
#[case::wrap_key_and_sign(TestCase::wrap_key_and_sign)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn hsm_tests<F>(#[context] ctx: Context, #[case] test: F)
where
    F: AsyncFnOnce(TestCase<Pkcs11Hsm>) -> TestCase<Pkcs11Hsm>,
{
    let test_case = TestCase::new("hsm.toml", ctx.description.unwrap());
    let test_case = test(test_case).await;
    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}
