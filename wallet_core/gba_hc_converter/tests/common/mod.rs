use std::{env, fs, path::PathBuf};

use aes_gcm::{Aes256Gcm, KeyInit};
use rand_core::OsRng;
use tempfile::TempDir;

use gba_hc_converter::{
    gba::encryption::{encrypt_bytes_to_dir, name_to_encoded_hash, HmacSha256},
    settings::SymmetricKey,
};

fn manifest_path() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap()
}

fn xml_resources_path() -> PathBuf {
    manifest_path().join("tests/resources")
}

pub async fn read_file(name: &str) -> String {
    tokio::fs::read_to_string(xml_resources_path().join(name))
        .await
        .unwrap()
}

pub async fn encrypt_xmls() -> (SymmetricKey, SymmetricKey, TempDir) {
    let encryption_key = SymmetricKey::new(Aes256Gcm::generate_key(OsRng).to_vec());
    let hmac_key = SymmetricKey::new(HmacSha256::generate_key(OsRng).to_vec());

    let temp_path = tempfile::tempdir().unwrap();

    let paths = fs::read_dir(xml_resources_path().join("gba")).unwrap();
    for path in paths {
        let entry = path.unwrap();
        if entry.file_type().unwrap().is_file() {
            let file = entry.path();
            let filename = String::from(file.file_stem().unwrap().to_str().unwrap());
            let content = tokio::fs::read(file).await.unwrap();

            let name = name_to_encoded_hash(&filename, &hmac_key);

            encrypt_bytes_to_dir(encryption_key.key::<Aes256Gcm>(), &content, temp_path.path(), &name)
                .await
                .unwrap();
        }
    }

    (encryption_key, hmac_key, temp_path)
}
