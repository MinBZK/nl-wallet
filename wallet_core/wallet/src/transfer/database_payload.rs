use derive_more::AsRef;
use derive_more::Constructor;
use josekit::JoseError;
use josekit::jwe::ECDH_ES_A256KW;
use josekit::jwe::JweHeader;
use josekit::jwe::enc::aesgcm::AesgcmJweEncryption;
use josekit::jwk::Jwk;
use josekit::jwt::JwtPayload;

use crate::storage::DatabaseExport;

const DATABASE_CLAIM: &str = "wallet_database";

#[derive(Debug, thiserror::Error)]
pub enum DatabasePayloadError {
    #[error("jose error: {0}")]
    Jose(#[from] JoseError),

    #[error("error serializing database export: {0}")]
    Serialization(#[source] serde_json::Error),

    #[error("error deserializing database export: {0}")]
    Deserialization(#[source] serde_json::Error),

    #[error("missing database claim")]
    MissingDatabaseClaim,
}

#[derive(Constructor, AsRef, PartialEq, Eq)]
pub struct WalletDatabasePayload(DatabaseExport);

impl WalletDatabasePayload {
    pub fn encrypt(self, public_key: &Jwk) -> Result<String, DatabasePayloadError> {
        let mut header = JweHeader::new();
        header.set_token_type("JWT");
        header.set_content_encryption(AesgcmJweEncryption::A256gcm.name());

        let mut payload = JwtPayload::new();
        payload.set_claim(
            DATABASE_CLAIM,
            Some(serde_json::to_value(self.0).map_err(DatabasePayloadError::Serialization)?),
        )?;

        let encrypter = ECDH_ES_A256KW.encrypter_from_jwk(public_key)?;
        let payload = josekit::jwt::encode_with_encrypter(&payload, &header, &encrypter)?;

        Ok(payload)
    }

    #[cfg(test)] // TODO: remove test annotation in PVW-4598
    pub fn decrypt(jwt: &str, private_key: &Jwk) -> Result<Self, DatabasePayloadError> {
        let decrypter = ECDH_ES_A256KW.decrypter_from_jwk(private_key)?;

        let (jwt_payload, _jwe_header) = josekit::jwt::decode_with_decrypter(jwt, &decrypter)?;

        let mut claims: serde_json::Map<String, serde_json::Value> = jwt_payload.into();

        let Some(serialized_database_export) = claims.remove(DATABASE_CLAIM) else {
            return Err(DatabasePayloadError::MissingDatabaseClaim);
        };

        let decrypted_database_export: DatabaseExport =
            serde_json::from_value(serialized_database_export).map_err(DatabasePayloadError::Deserialization)?;

        Ok(Self(decrypted_database_export))
    }
}

#[cfg(test)]
mod test {
    use josekit::jwk::KeyPair;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;

    use crypto::utils::random_bytes;

    use crate::storage::DatabaseExport;
    use crate::storage::test::SqlCipherKey;
    use crate::transfer::database_payload::WalletDatabasePayload;

    #[test]
    fn test_encrypt_decrypt() {
        let database_export_bytes = random_bytes(32);
        let database_export_key = SqlCipherKey::new_random_with_salt();
        let database_export = DatabaseExport::new(database_export_key, database_export_bytes.clone());
        let database_payload = WalletDatabasePayload::new(database_export);

        let key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let jwt = database_payload.encrypt(&key_pair.to_jwk_public_key()).unwrap();

        let decrypted = WalletDatabasePayload::decrypt(&jwt, &key_pair.to_jwk_private_key()).unwrap();

        assert!(
            decrypted == WalletDatabasePayload::new(DatabaseExport::new(database_export_key, database_export_bytes))
        );
    }
}
