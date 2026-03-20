use derive_more::AsRef;
use derive_more::Constructor;
use derive_more::Into;

use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweDecrypterError;
use jwe::decryption::JweSecretKey;
use jwe::encryption::JweCompression;
use jwe::encryption::JweEncrypter;
use jwe::encryption::JweEncrypterError;
use jwe::encryption::JwePublicKey;

use crate::storage::DatabaseExport;

#[derive(Constructor, AsRef, Into, PartialEq, Eq)]
pub struct WalletDatabasePayload(DatabaseExport);

impl WalletDatabasePayload {
    pub fn encrypt(&self, public_key: JwePublicKey) -> Result<String, JweEncrypterError> {
        let Self(export) = self;

        let encryper = JweEncrypter::from(public_key);
        let jwe = encryper.encrypt(
            export,
            EncryptionAlgorithm::A256Gcm,
            None,
            None,
            JweCompression::Deflate,
        )?;

        Ok(jwe)
    }

    pub fn decrypt(jwe: &str, secret_key: &JweSecretKey) -> Result<Self, JweDecrypterError> {
        let decrypter = JweDecrypter::from_secret_key(secret_key);

        let export = decrypter.decrypt(jwe)?;

        Ok(Self(export))
    }
}

#[cfg(test)]
mod test {
    use crypto::utils::random_bytes;
    use jwe::algorithm::EcdhAlgorithm;
    use jwe::decryption::JweSecretKey;

    use crate::storage::DatabaseExport;
    use crate::storage::test::SqlCipherKey;
    use crate::transfer::database_payload::WalletDatabasePayload;

    #[test]
    fn test_encrypt_decrypt() {
        let database_export_bytes = random_bytes(256);
        let database_export_key = SqlCipherKey::new_random_with_salt();
        let database_export = DatabaseExport::new(database_export_key, database_export_bytes.clone());
        let database_payload = WalletDatabasePayload::new(database_export);

        let secret_key = JweSecretKey::new_random(None, EcdhAlgorithm::EcdhEsA256kw);

        let jwe = database_payload.encrypt(secret_key.to_jwe_public_key()).unwrap();

        let decrypted = WalletDatabasePayload::decrypt(&jwe, &secret_key).unwrap();

        assert!(decrypted == database_payload);
    }
}
