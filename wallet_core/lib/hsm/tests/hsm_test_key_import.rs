use hsm::service::Pkcs11Hsm;
use hsm::test::TestCase;
use hsm::test::execute_hsm_test;
use rstest::Context;
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[case::aes_cmac_test_vectors(TestCase::aes_cmac_test_vectors)]
#[case::aes_ctr_test_vectors(TestCase::aes_ctr_test_vectors)]
#[case::aes_siv_encrypt_test_vectors(TestCase::aes_siv_encrypt_test_vectors)]
#[case::aes_siv_decrypt_test_vectors(TestCase::aes_siv_decrypt_test_vectors)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn hsm_key_importing_tests<F>(#[context] ctx: Context, #[case] test: F)
where
    F: AsyncFnOnce(TestCase<Pkcs11Hsm>) -> TestCase<Pkcs11Hsm>,
{
    execute_hsm_test(ctx.description.unwrap().to_string(), test).await
}
