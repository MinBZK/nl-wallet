use std::path::{Path, PathBuf};

use anyhow::{Ok, Result};
use platform_support::{hw_keystore::PlatformEncryptionKey, utils::PlatformUtilities};
use tokio::fs;
use wallet_common::utils::random_string;

const PASSWORD_LENGTH: usize = 32;

pub async fn get_or_create_password<K: PlatformEncryptionKey, U: PlatformUtilities>(alias: &str) -> Result<String> {
    // Path to password file will be "<storage_path>/<alias>.pass",
    // it will be encrypted with a key named "passwordfile_<alias>".
    let path = path_for_password::<U>(alias)?;
    let encryption_key = K::new(&format!("passwordfile_{}", alias));

    // Decrypt file at path, create password and write to file if needed.
    get_or_create_encrypted_file_contents(path.as_path(), &encryption_key, || random_string(PASSWORD_LENGTH)).await
}

pub async fn delete_password<U: PlatformUtilities>(alias: &str) -> Result<bool> {
    let path = path_for_password::<U>(alias)?;
    let remove_result = fs::remove_file(&path).await;

    // Return true if the delete did not result in an error.
    Ok(remove_result.is_ok())
}

fn path_for_password<U: PlatformUtilities>(alias: &str) -> Result<PathBuf> {
    let storage_path = U::storage_path()?;
    let path = storage_path.join(format!("{}.pass", alias));

    Ok(path)
}

async fn get_or_create_encrypted_file_contents(
    path: &Path,
    encryption_key: &impl PlatformEncryptionKey,
    default: impl FnOnce() -> String,
) -> Result<String> {
    // If no file at the path exsits, call the default closure to get the desired contents,
    // ecnrypt it and write it to a new file at the path.
    if !fs::try_exists(path).await? {
        let contents = default();
        write_encrypted_file(path, contents.as_bytes(), encryption_key).await?;

        return Ok(contents);
    }

    // Otherwise, decrypt the file and return its contents as a String
    let contents = String::from_utf8(read_encrypted_file(path, encryption_key).await?)?;

    Ok(contents)
}

async fn write_encrypted_file(path: &Path, contents: &[u8], encryption_key: &impl PlatformEncryptionKey) -> Result<()> {
    // Encrypt the contents as bytes and write to a new file at the path.
    let encrypted_contents = encryption_key.encrypt(contents)?;
    fs::write(path, &encrypted_contents).await?;

    Ok(())
}

async fn read_encrypted_file(path: &Path, encryption_key: &impl PlatformEncryptionKey) -> Result<Vec<u8>> {
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
    fn create_temporary_file_path() -> Result<TempPath> {
        let path = NamedTempFile::new()?.into_temp_path();
        std::fs::remove_file(&path)?;

        Ok(path)
    }

    #[tokio::test]
    async fn test_read_and_write_encrypted_file() -> Result<()> {
        let path = create_temporary_file_path()?;
        let contents = "This will be encrypted in a file.";
        let encryption_key = SoftwareEncryptionKey::new("test_read_and_write_encrypted_file");

        // encrypt and decrypt a file, read encrypted file manually.
        write_encrypted_file(&path, contents.as_bytes(), &encryption_key).await?;
        let encrypted_contents = fs::read(path.to_path_buf()).await?;
        let decrypted_contents = read_encrypted_file(&path, &encryption_key).await?;

        assert!(!encrypted_contents.is_empty());
        assert!(!decrypted_contents.is_empty());
        assert_ne!(encrypted_contents, contents.as_bytes());
        assert_eq!(decrypted_contents, contents.as_bytes());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_encrypted_file_contents() -> Result<()> {
        let path = create_temporary_file_path()?;
        let encryption_key = SoftwareEncryptionKey::new("get_or_create_encrypted_file_contents");

        let contents = "This will be encrypted in a file.";
        let default_counter = RefCell::new(0);
        let default = || {
            *default_counter.borrow_mut() += 1;

            contents.to_string()
        };

        // This should create a new file and call the default closure.
        let contents1 = get_or_create_encrypted_file_contents(&path, &encryption_key, default).await?;

        assert_eq!(contents1, contents);
        assert_eq!(*default_counter.borrow(), 1);

        // This should read the encrypted file from disk and not call the default closure.
        let contents2 = get_or_create_encrypted_file_contents(&path, &encryption_key, default).await?;

        assert_eq!(contents2, contents);
        assert_eq!(*default_counter.borrow(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_password() -> Result<()> {
        let alias1 = "test_get_or_create_password1".to_string();
        let alias2 = "test_get_or_create_password2".to_string();

        // Make sure we start with a clean slate.
        delete_password::<SoftwareUtilities>(&alias1).await?;
        delete_password::<SoftwareUtilities>(&alias2).await?;

        // Create three passwords, two of them with the same alias.
        let password1 = get_or_create_password::<SoftwareEncryptionKey, SoftwareUtilities>(&alias1).await?;
        let password2 = get_or_create_password::<SoftwareEncryptionKey, SoftwareUtilities>(&alias2).await?;
        let password1_again = get_or_create_password::<SoftwareEncryptionKey, SoftwareUtilities>(&alias1).await?;

        assert!(!password1.is_empty());
        assert!(!password2.is_empty());
        assert_ne!(password1, password2);
        assert_eq!(password1, password1_again);

        // Cleanup after ourselves.
        delete_password::<SoftwareUtilities>(&alias1).await?;
        delete_password::<SoftwareUtilities>(&alias2).await?;

        Ok(())
    }
}
