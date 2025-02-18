use std::fs;
use std::path::PathBuf;

use aes_gcm::Aes256Gcm;
use aes_gcm::KeyInit;
use rand_core::OsRng;
use tempfile::TempDir;

use wallet_common::utils;

use gba_hc_converter::gba::encryption::encrypt_bytes_to_dir;
use gba_hc_converter::gba::encryption::HmacSha256;
use gba_hc_converter::settings::SymmetricKey;

fn xml_resources_path() -> PathBuf {
    utils::prefix_local_path("tests/resources".as_ref()).into_owned()
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

            encrypt_bytes_to_dir(
                encryption_key.key::<Aes256Gcm>(),
                hmac_key.key::<HmacSha256>(),
                &content,
                temp_path.path(),
                &filename,
            )
            .await
            .unwrap();
        }
    }

    (encryption_key, hmac_key, temp_path)
}
