use rstest::Context;
use rstest::rstest;

use hsm::model::mock::MockPkcs11Client;
use hsm::service::HsmError;
use hsm::test::TestCase;

#[rstest]
#[case::sign_sha256_hmac(TestCase::sign_sha256_hmac)]
#[case::sign_ecdsa(TestCase::sign_ecdsa)]
#[case::encrypt_decrypt(TestCase::encrypt_decrypt)]
#[case::encrypt_decrypt_verifying_key(TestCase::encrypt_decrypt_verifying_key)]
// #[case::wrap_key_and_sign(TestCase::wrap_key_and_sign)] // TODO: generate aes key is unsupported for Mock
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn hsm_mock_tests<F>(#[context] ctx: Context, #[case] test: F)
where
    F: AsyncFnOnce(TestCase<MockPkcs11Client<HsmError>>) -> TestCase<MockPkcs11Client<HsmError>>,
{
    let test_case = TestCase::mock(ctx.description.unwrap());
    test(test_case).await;
}
