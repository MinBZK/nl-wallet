use std::{
    io, panic,
    path::{Path, PathBuf},
};

use tokio::{
    fs,
    task::{self, JoinError},
};

use platform_support::{
    hw_keystore::{HardwareKeyStoreError, PlatformEncryptionKey},
    utils::{PlatformUtilities, UtilitiesError},
};
use wallet_common::utils::random_bytes;

const KEY_IDENTIFIER_PREFIX: &str = "keyfile_";

#[derive(Debug, thiserror::Error)]
pub enum KeyFileError {
    #[error("key file I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("key file platform join error: {0}")]
    Join(#[from] JoinError),
    #[error("key file platform utilities error: {0}")]
    PlatformUtilities(#[from] UtilitiesError),
    #[error("key file platform key store error: {0}")]
    PlatformKeyStore(#[from] HardwareKeyStoreError),
}

pub async fn get_or_create_key_file<K: PlatformEncryptionKey, U: PlatformUtilities>(
    alias: &str,
    byte_length: usize,
) -> Result<Vec<u8>, KeyFileError> {
    // Path to key file will be "<storage_path>/<alias>.key",
    // it will be encrypted with a key named "keyfile_<alias>".
    let path = path_for_key_file::<U>(alias).await?;
    let encryption_key = K::new(&format!("{}{}", KEY_IDENTIFIER_PREFIX, alias));

    // Decrypt file at path, create key and write to file if needed.
    get_or_create_encrypted_file_contents(path.as_path(), &encryption_key, || random_bytes(byte_length)).await
}

pub async fn delete_key_file<U: PlatformUtilities>(alias: &str) -> Result<(), KeyFileError> {
    let path = path_for_key_file::<U>(alias).await?;
    // Ignore any errors when removing the file,
    // as we do not want this to propagate.
    let _ = fs::remove_file(&path).await;

    Ok(())
}

async fn path_for_key_file<U: PlatformUtilities>(alias: &str) -> Result<PathBuf, KeyFileError> {
    let storage_path = task::spawn_blocking(|| U::storage_path())
        .await
        .map_err(|e| match e.try_into_panic() {
            Ok(panic) => panic::resume_unwind(panic),
            Err(e) => e,
        })??;
    let path = storage_path.join(format!("{}.key", alias));

    Ok(path)
}

async fn get_or_create_encrypted_file_contents(
    path: &Path,
    encryption_key: &impl PlatformEncryptionKey,
    default: impl FnOnce() -> Vec<u8>,
) -> Result<Vec<u8>, KeyFileError> {
    // If no file at the path exsits, call the default closure to get the desired contents,
    // ecnrypt it and write it to a new file at the path.
    if !fs::try_exists(path).await? {
        let contents = default();
        write_encrypted_file(path, &contents, encryption_key).await?;

        return Ok(contents);
    }

    // Otherwise, decrypt the file and return its contents
    read_encrypted_file(path, encryption_key).await
}

async fn write_encrypted_file(
    path: &Path,
    contents: &[u8],
    encryption_key: &impl PlatformEncryptionKey,
) -> Result<(), KeyFileError> {
    // Encrypt the contents as bytes and write to a new file at the path.
    let encrypted_contents = encryption_key.encrypt(contents)?;
    fs::write(path, &encrypted_contents).await?;

    Ok(())
}

async fn read_encrypted_file(
    path: &Path,
    encryption_key: &impl PlatformEncryptionKey,
) -> Result<Vec<u8>, KeyFileError> {
    // Decrypt the bytes of a file at the path.
    let contents = fs::read(path).await?;
    let decrypted_contents = encryption_key.decrypt(&contents)?;

    Ok(decrypted_contents)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use platform_support::{
        hw_keystore::{software::SoftwareEncryptionKey, ConstructableWithIdentifier},
        utils::software::SoftwareUtilities,
    };
    use tempfile::{NamedTempFile, TempPath};

    use super::*;

    // Create a temporary file path by deleting newly created NamedTempFile.
    fn create_temporary_file_path() -> TempPath {
        let path = NamedTempFile::new()
            .expect("Could not create new temporary file")
            .into_temp_path();
        std::fs::remove_file(&path).expect("Could not remove file");

        path
    }

    #[tokio::test]
    async fn test_read_and_write_encrypted_file() {
        let path = create_temporary_file_path();
        let contents = "This will be encrypted in a file.";
        let encryption_key = SoftwareEncryptionKey::new("test_read_and_write_encrypted_file");

        // encrypt and decrypt a file, read encrypted file manually.
        write_encrypted_file(&path, contents.as_bytes(), &encryption_key)
            .await
            .expect("Could not write encrypted file");
        let encrypted_contents = fs::read(path.to_path_buf())
            .await
            .expect("Could not read encrypted file");
        let decrypted_contents = read_encrypted_file(&path, &encryption_key)
            .await
            .expect("Could not read and decrypt encrypted file");

        assert!(!encrypted_contents.is_empty());
        assert!(!decrypted_contents.is_empty());
        assert_ne!(encrypted_contents, contents.as_bytes());
        assert_eq!(decrypted_contents, contents.as_bytes());
    }

    #[tokio::test]
    async fn test_get_or_create_encrypted_file_contents() {
        let path = create_temporary_file_path();
        let encryption_key = SoftwareEncryptionKey::new("get_or_create_encrypted_file_contents");

        let contents = "This will be encrypted in a file.";
        let default_counter = RefCell::new(0);
        let default = || {
            *default_counter.borrow_mut() += 1;

            contents.as_bytes().to_vec()
        };

        // This should create a new file and call the default closure.
        let contents1 = get_or_create_encrypted_file_contents(&path, &encryption_key, default)
            .await
            .expect("Could not create encrypted file");

        assert_eq!(contents1, contents.as_bytes());
        assert_eq!(*default_counter.borrow(), 1);

        // This should read the encrypted file from disk and not call the default closure.
        let contents2 = get_or_create_encrypted_file_contents(&path, &encryption_key, default)
            .await
            .expect("Could not read encrypted file");

        assert_eq!(contents2, contents.as_bytes());
        assert_eq!(*default_counter.borrow(), 1);
    }

    #[tokio::test]
    async fn test_get_or_create_key() {
        let alias1 = "test_get_or_create_key1".to_string();
        let alias2 = "test_get_or_create_key2".to_string();
        let byte_length: usize = 48;

        // Make sure we start with a clean slate.
        delete_key_file::<SoftwareUtilities>(&alias1).await.unwrap();
        delete_key_file::<SoftwareUtilities>(&alias2).await.unwrap();

        // Create three keys, two of them with the same alias.
        let key1 = get_or_create_key_file::<SoftwareEncryptionKey, SoftwareUtilities>(&alias1, byte_length)
            .await
            .expect("Could not create key file");
        let key2 = get_or_create_key_file::<SoftwareEncryptionKey, SoftwareUtilities>(&alias2, byte_length)
            .await
            .expect("Could not create key file");
        let key1_again = get_or_create_key_file::<SoftwareEncryptionKey, SoftwareUtilities>(&alias1, byte_length)
            .await
            .expect("Could not get key file");

        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
        assert_ne!(key1, key2);
        assert_eq!(key1, key1_again);

        // Cleanup after ourselves.
        delete_key_file::<SoftwareUtilities>(&alias1).await.unwrap();
        delete_key_file::<SoftwareUtilities>(&alias2).await.unwrap();
    }
}
