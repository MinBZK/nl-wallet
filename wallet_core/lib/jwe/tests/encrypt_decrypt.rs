use std::assert_matches;

use jwe::algorithm::CompressionAlgorithm;
use jwe::algorithm::EcdhAlgorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::AllowedDecompression;
use jwe::decryption::ExpectedEncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweEcdhSecretKey;
use jwe::encryption::JweEncrypter;
use jwe::encryption::JwePublicKey;
use jwe::error::JweDecryptionError;
use jwe::error::JweJsonDecryptionError;
use jwe::error::JweStringDecryptionError;
use jwk_simple::Key;
use rstest::rstest;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestPayload {
    message: String,
    count: u64,
    numbers: Vec<i64>,
}

fn setup_receiver(kid: Option<String>) -> (JweEcdhSecretKey, Key) {
    let secret_key = JweEcdhSecretKey::new_random(kid, EcdhAlgorithm::EcdhEs);
    let jwk = Key::from(secret_key.to_jwe_public_key());

    (secret_key, jwk)
}

fn encrypt_jwe<T>(jwk: &Key, payload: &T, use_compression: Option<CompressionAlgorithm>) -> String
where
    T: Serialize,
{
    let public_key = JwePublicKey::try_from_jwk(jwk).expect("converting JWK to JwePublicKey should succeed");

    JweEncrypter::from(public_key)
        .encrypt_json(
            &payload,
            EncryptionAlgorithm::A256Gcm,
            Some(b"apu"),
            Some(b"apv"),
            use_compression,
        )
        .expect("encrypting payload to JWE should succeed")
}

#[rstest]
#[case::no_compression_deny_decompression(None, AllowedDecompression::None)]
#[case::no_compression_allow_decompression(None, AllowedDecompression::Deflate)]
#[case::use_compression_allow_decompression(Some(CompressionAlgorithm::Deflate), AllowedDecompression::Deflate)]
// Using compression while decompression is disallowed during decryption is an error, which is tested below.
fn test_encrypt_decrypt_ok(
    #[case] use_compression: Option<CompressionAlgorithm>,
    #[case] allowed_decompression: AllowedDecompression,
    #[values(None, Some("key_id"))] kid: Option<&str>,
    #[values(false, true)] expect_encryption_algorithm: bool,
) {
    let payload = TestPayload {
        message: "This is a plaintext message.".to_string(),
        count: 321,
        numbers: vec![1, 2, 3, -1, -2, 3],
    };

    // Receiving side.
    let (secret_key, jwk) = setup_receiver(kid.map(str::to_string));

    // Sending side.
    let jwe = encrypt_jwe(&jwk, &payload, use_compression);

    // Receiving side again.
    let expected_algorithm = if expect_encryption_algorithm {
        ExpectedEncryptionAlgorithm::Algorithms(&[EncryptionAlgorithm::A192Gcm, EncryptionAlgorithm::A256Gcm])
    } else {
        ExpectedEncryptionAlgorithm::Any
    };
    let decrypted_payload = JweDecrypter::from_ecdh_secret_key(&secret_key)
        .decrypt_json::<TestPayload>(&jwe, expected_algorithm, allowed_decompression)
        .expect("decrypting payload from JWE should succeed");

    assert_eq!(decrypted_payload, payload);
}

#[test]
fn test_encrypt_decrypt_id_mismatch() {
    let (secret_key, jwk) = setup_receiver(Some("key_id".to_string()));

    // The sender should not include a different kid value in the JWE.
    let jwk_wrong_kid = jwk.clone().with_kid("wrong_key_id".to_string());
    let jwe = encrypt_jwe(&jwk_wrong_kid, &(), Some(CompressionAlgorithm::Deflate));
    let error = JweDecrypter::from_ecdh_secret_key(&secret_key)
        .decrypt_json::<()>(&jwe, ExpectedEncryptionAlgorithm::Any, AllowedDecompression::Deflate)
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweJsonDecryptionError::JweDecryption(JweDecryptionError::Decryption(_))
    );

    // The sender should not omit the kid value from the JWE.
    let jwk_no_kid = Key::new(jwk.params().clone())
        .with_alg(jwk.alg().cloned().unwrap())
        .with_use(jwk.key_use().cloned().unwrap());
    let jwe = encrypt_jwe(&jwk_no_kid, &(), Some(CompressionAlgorithm::Deflate));
    let error = JweDecrypter::from_ecdh_secret_key(&secret_key)
        .decrypt_json::<()>(&jwe, ExpectedEncryptionAlgorithm::Any, AllowedDecompression::Deflate)
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweJsonDecryptionError::JweDecryption(JweDecryptionError::Decryption(_))
    );
}

#[test]
fn test_encrypt_decrypt_compression_not_allowed() {
    let (secret_key, jwk) = setup_receiver(None);
    let decrypter = JweDecrypter::from_ecdh_secret_key(&secret_key);

    let jwe = encrypt_jwe(&jwk, &(), Some(CompressionAlgorithm::Deflate));

    let error = decrypter
        .decrypt_json::<()>(&jwe, ExpectedEncryptionAlgorithm::Any, AllowedDecompression::None)
        .expect_err("decrypting compressed JWE as JSON should fail");

    assert_matches!(
        &error,
        JweJsonDecryptionError::JweDecryption(JweDecryptionError::Decryption(_))
    );

    let error = decrypter
        .decrypt_string(&jwe, ExpectedEncryptionAlgorithm::Any, AllowedDecompression::None)
        .expect_err("decrypting compressed JWE as string should fail");

    assert_matches!(
        &error,
        JweStringDecryptionError::JweDecryption(JweDecryptionError::Decryption(_))
    );
}

#[test]
fn test_encrypt_decrypt_unexpected_encryption_algorithm() {
    let (secret_key, jwk) = setup_receiver(None);

    // Decryption should fail if A128GCM is not used.
    let jwe = encrypt_jwe(&jwk, &(), Some(CompressionAlgorithm::Deflate));
    let error = JweDecrypter::from_ecdh_secret_key(&secret_key)
        .decrypt_json::<()>(
            &jwe,
            ExpectedEncryptionAlgorithm::Algorithms(&[EncryptionAlgorithm::A128Gcm]),
            AllowedDecompression::Deflate,
        )
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweJsonDecryptionError::JweDecryption(JweDecryptionError::UnexpectedEncryptionAlgorithm { received, expected })
            if received == "A256GCM" && expected == vec![EncryptionAlgorithm::A128Gcm]
    );

    // Decryption should always fail if no encryption algorithm is expected.
    let error = JweDecrypter::from_ecdh_secret_key(&secret_key)
        .decrypt_json::<()>(
            &jwe,
            ExpectedEncryptionAlgorithm::Algorithms(&[]),
            AllowedDecompression::Deflate,
        )
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweJsonDecryptionError::JweDecryption(JweDecryptionError::UnexpectedEncryptionAlgorithm { received, expected })
            if received == "A256GCM" && expected == vec![]
    );
}
