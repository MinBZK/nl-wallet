use std::path::Path;

use aes_gcm::{
    aead::{Aead, Nonce},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use hmac::{Hmac, Mac};
use rand_core::OsRng;
use sha2::Sha256;

use crate::{gba::error::Error, settings::SymmetricKey};

pub type HmacSha256 = Hmac<Sha256>;

pub async fn encrypt_bytes_to_dir(key: &Key<Aes256Gcm>, bytes: &[u8], dir: &Path, basename: &str) -> Result<(), Error> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, bytes)?;

    tokio::fs::write(dir.join(format!("{}.aes", basename)), ciphertext).await?;
    tokio::fs::write(dir.join(format!("{}.nonce", basename)), nonce).await?;

    Ok(())
}

pub async fn decrypt_bytes_from_dir(
    decryption_key: &Key<Aes256Gcm>,
    dir: &Path,
    basename: &str,
) -> Result<Option<Vec<u8>>, Error> {
    let encrypted_file = dir.join(format!("{basename}.aes"));
    let nonce_file = dir.join(format!("{basename}.nonce"));
    if encrypted_file.exists() && nonce_file.exists() {
        let encrypted_xml = tokio::fs::read(encrypted_file).await?;
        let nonce_str = tokio::fs::read(nonce_file).await?;

        let cipher = Aes256Gcm::new(decryption_key);
        let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_str.as_slice());
        let bytes = cipher.decrypt(nonce, encrypted_xml.as_slice())?;
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}

fn name_to_mac(name: &str, hmac_key: &Key<HmacSha256>) -> HmacSha256 {
    let mut mac = <HmacSha256 as Mac>::new(hmac_key);
    mac.update(name.as_bytes());
    mac
}

pub fn name_to_encoded_hash(name: &str, hmac_key: &SymmetricKey) -> String {
    let mac = name_to_mac(name, hmac_key.key::<HmacSha256>());
    let authentication_code = mac.finalize().into_bytes();
    hex::encode(authentication_code)
}

pub fn verify_name(name: &str, authentication_code: &str, hmac_key: &Key<HmacSha256>) -> Result<bool, Error> {
    let mac = name_to_mac(name, hmac_key);
    Ok(mac.verify_slice(&hex::decode(authentication_code)?).is_ok())
}

#[cfg(test)]
mod tests {
    use wallet_common::utils::random_bytes;

    use crate::{
        gba::encryption::{name_to_encoded_hash, verify_name, HmacSha256},
        settings::SymmetricKey,
    };

    #[test]
    fn encode_to_hash_and_verify() {
        let name = "aodfijaslfasfaefljas";
        let key = SymmetricKey::new(random_bytes(64));
        let hash = name_to_encoded_hash(name, &key);

        assert_eq!(
            hash,
            name_to_encoded_hash(name, &key),
            "should generate identical hash for identical input"
        );

        assert!(verify_name(name, &hash, key.key::<HmacSha256>()).unwrap());
    }
}
