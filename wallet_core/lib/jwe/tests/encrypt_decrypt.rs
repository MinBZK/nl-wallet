use assert_matches::assert_matches;
use jwe::algorithm::EcdhAlgorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweDecrypterError;
use jwe::decryption::JweSecretKey;
use jwe::encryption::JweEncrypter;
use jwe::encryption::JwePublicKey;
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

fn setup_receiver(kid: Option<String>) -> (JweSecretKey, Key) {
    let secret_key = JweSecretKey::new_random(kid, EcdhAlgorithm::EcdhEs);
    let jwk = Key::from(secret_key.to_jwe_public_key());

    (secret_key, jwk)
}

fn encrypt_jwe<T>(jwk: &Key, payload: &T) -> String
where
    T: Serialize,
{
    let public_key = JwePublicKey::try_from_jwk(jwk).expect("converting JWK to JwePublicKey should succeed");

    JweEncrypter::from(public_key)
        .encrypt(&payload, EncryptionAlgorithm::A256Gcm, Some(b"apu"), Some(b"apv"))
        .expect("encrypting payload to JWE should succeed")
}

#[rstest]
fn test_encrypt_decrypt_ok(#[values(None, Some("key_id"))] kid: Option<&str>) {
    let payload = TestPayload {
        message: "This is a plaintext message.".to_string(),
        count: 321,
        numbers: vec![1, 2, 3, -1, -2, 3],
    };

    // Receiving side.
    let (secret_key, jwk) = setup_receiver(kid.map(str::to_string));

    // Sending side.
    let jwe = encrypt_jwe(&jwk, &payload);

    // Receiving side again.
    let decrypted_payload = JweDecrypter::from_secret_key(&secret_key)
        .decrypt::<TestPayload>(&jwe)
        .expect("decrypting payload from JWE should succeed");

    assert_eq!(decrypted_payload, payload);
}

#[test]
fn test_encrypt_decrypt_id_mismatch() {
    let (secret_key, jwk) = setup_receiver(Some("key_id".to_string()));

    // The sender should not include a different kid value in the JWE.
    let jwk_wrong_kid = jwk.clone().with_kid("wrong_key_id".to_string());
    let jwe = encrypt_jwe(&jwk_wrong_kid, &());
    let error = JweDecrypter::from_secret_key(&secret_key)
        .decrypt::<()>(&jwe)
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweDecrypterError::IdMismatch(expected_kid, Some(received_kid))
            if &expected_kid == "key_id" && received_kid == "wrong_key_id"
    );

    // The sender should not omit the kid value from the JWE.
    let jwk_no_kid = Key::new(jwk.params().clone())
        .with_alg(jwk.alg().cloned().unwrap())
        .with_use(jwk.key_use().cloned().unwrap());
    let jwe = encrypt_jwe(&jwk_no_kid, &());
    let error = JweDecrypter::from_secret_key(&secret_key)
        .decrypt::<()>(&jwe)
        .expect_err("decrypting payload from JWE should fail");

    assert_matches!(
        error,
        JweDecrypterError::IdMismatch(expected_kid, None) if &expected_kid == "key_id"
    );
}
