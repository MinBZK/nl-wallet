use std::path::Path;

use aes_gcm::{
    aead::{Aead, Nonce},
    aes::Aes256,
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use rand_core::OsRng;

use crate::gba::error::Error;

pub async fn encrypt_bytes_to_dir(key: &Key<Aes256>, bytes: &[u8], dir: &Path, basename: &str) -> Result<(), Error> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, bytes)?;

    tokio::fs::write(dir.join(format!("{}.aes", basename)), ciphertext).await?;
    tokio::fs::write(dir.join(format!("{}.nonce", basename)), nonce).await?;

    Ok(())
}

pub async fn decrypt_bytes_from_dir(key: &Key<Aes256>, dir: &Path, basename: &str) -> Result<Option<Vec<u8>>, Error> {
    let encrypted_file = dir.join(format!("{basename}.aes"));
    let nonce_file = dir.join(format!("{basename}.nonce"));
    if encrypted_file.exists() && nonce_file.exists() {
        let encrypted_xml = tokio::fs::read(encrypted_file).await?;
        let nonce_str = tokio::fs::read(nonce_file).await?;

        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_str.as_slice());
        let bytes = cipher.decrypt(nonce, encrypted_xml.as_slice())?;
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}
