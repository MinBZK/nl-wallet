use std::{
    future::Future,
    path::{Path, PathBuf},
};

use aes_gcm::{
    aead::{Aead, Nonce},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use hmac::{Hmac, Mac};
use rand_core::OsRng;
use sha2::Sha256;
use tokio::fs::DirEntry;
use tracing::debug;

use crate::gba::error::Error;

pub type HmacSha256 = Hmac<Sha256>;

const AES256GCM_NONCE_SIZE: usize = 12;

pub async fn encrypt_bytes_to_dir(
    encryption_key: &Key<Aes256Gcm>,
    hmac_key: &Key<HmacSha256>,
    bytes: &[u8],
    output_path: &Path,
    basename: &str,
) -> Result<(), Error> {
    debug!("encrypting bytes to dir");
    let ciphertext = encrypt_bytes(encryption_key, bytes)?;
    tokio::fs::write(filename(hmac_key, output_path, basename), ciphertext).await?;
    Ok(())
}

pub async fn decrypt_bytes_from_dir(
    decryption_key: &Key<Aes256Gcm>,
    hmac_key: &Key<HmacSha256>,
    input_path: &Path,
    basename: &str,
) -> Result<Option<Vec<u8>>, Error> {
    let filename = filename(hmac_key, input_path, basename);
    if filename.exists() {
        let bytes = tokio::fs::read(filename).await?;
        let decrypted = decrypt_bytes(decryption_key, &bytes)?;
        debug!("decrypting bytes from dir");
        Ok(Some(decrypted))
    } else {
        debug!("file to decrypt not found");
        Ok(None)
    }
}

pub async fn count_files_in_dir(path: &Path) -> Result<u64, Error> {
    let count = iterate_encrypted_files(path, |_| async { Ok(()) }).await?;
    Ok(count)
}

pub async fn clear_files_in_dir(path: &Path) -> Result<u64, Error> {
    let count = iterate_encrypted_files(
        path,
        |entry| async move { Ok(tokio::fs::remove_file(entry.path()).await?) },
    )
    .await?;
    Ok(count)
}

async fn iterate_encrypted_files<F, Fut, Out>(path: &Path, f: F) -> Result<u64, Error>
where
    F: Fn(DirEntry) -> Fut,
    Fut: Future<Output = Result<Out, Error>>,
{
    let mut count = 0;
    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() && entry.path().extension().is_some_and(|ext| ext == "aes") {
            f(entry).await?;
            count += 1;
        }
    }
    Ok(count)
}

fn filename(hmac_key: &Key<HmacSha256>, path: &Path, name: &str) -> PathBuf {
    let hmac = name_to_encoded_hash(name, hmac_key);
    path.join(format!("{}.aes", &hmac))
}

fn encrypt_bytes(key: &Key<Aes256Gcm>, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut ciphertext = cipher.encrypt(&nonce, bytes)?;

    let mut result = nonce.as_slice().to_vec();
    result.append(&mut ciphertext);

    Ok(result)
}

fn decrypt_bytes(decryption_key: &Key<Aes256Gcm>, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let (nonce, ciphertext) = bytes.split_at(AES256GCM_NONCE_SIZE);
    let nonce = Nonce::<Aes256Gcm>::from_slice(nonce);
    let cipher = Aes256Gcm::new(decryption_key);
    let bytes = cipher.decrypt(nonce, ciphertext)?;
    Ok(bytes)
}

fn name_to_mac(name: &str, hmac_key: &Key<HmacSha256>) -> HmacSha256 {
    let mut mac = <HmacSha256 as Mac>::new(hmac_key);
    mac.update(name.as_bytes());
    mac
}

fn name_to_encoded_hash(name: &str, hmac_key: &Key<HmacSha256>) -> String {
    let mac = name_to_mac(name, hmac_key);
    let authentication_code = mac.finalize().into_bytes();
    hex::encode(authentication_code)
}

pub fn verify_name(name: &str, authentication_code: &str, hmac_key: &Key<HmacSha256>) -> Result<bool, Error> {
    let mac = name_to_mac(name, hmac_key);
    Ok(mac.verify_slice(&hex::decode(authentication_code)?).is_ok())
}

#[cfg(test)]
mod tests {
    use wallet_common::utils::{random_bytes, random_string};

    use crate::{
        gba::encryption::{name_to_encoded_hash, verify_name, HmacSha256},
        settings::SymmetricKey,
    };

    #[test]
    fn encode_to_hash_and_verify() {
        let name = random_string(16);
        let key = SymmetricKey::new(random_bytes(64));
        let hash = name_to_encoded_hash(&name, key.key::<HmacSha256>());

        assert_eq!(
            hash,
            name_to_encoded_hash(&name, key.key::<HmacSha256>()),
            "should generate identical hash for identical input"
        );

        assert!(verify_name(&name, &hash, key.key::<HmacSha256>()).unwrap());
    }
}
