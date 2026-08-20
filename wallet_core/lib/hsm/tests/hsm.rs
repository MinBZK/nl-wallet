use hsm::service::Pkcs11Hsm;
use hsm::test::TestCase;
use hsm::test::execute_hsm_test;
use rstest::Context;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[case::sign_sha256_hmac(TestCase::sign_sha256_hmac)]
#[case::sign_ecdsa(TestCase::sign_ecdsa)]
#[case::encrypt_decrypt(TestCase::encrypt_decrypt)]
#[case::encrypt_decrypt_verifying_key(TestCase::encrypt_decrypt_verifying_key)]
#[case::wrap_key_and_sign(TestCase::wrap_key_and_sign)]
#[case::encrypt_ctr(TestCase::encrypt_ctr)]
#[case::cmac(TestCase::cmac)]
#[case::aes_siv(TestCase::aes_siv)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn hsm_tests<F>(#[context] ctx: Context, #[case] test: F)
where
    F: AsyncFnOnce(TestCase<Pkcs11Hsm>) -> TestCase<Pkcs11Hsm>,
{
    execute_hsm_test(ctx.description.unwrap().to_string(), test).await
}
