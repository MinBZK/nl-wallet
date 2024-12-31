use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use clio::CachedInput;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePrivateKey;
use pem::EncodeConfig;
use pem::LineEnding;
use pem::Pem;

use nl_wallet_mdoc::server_keys::generate::SelfSignedCa;

pub fn read_self_signed_ca(ca_crt_file: &CachedInput, ca_key_file: &CachedInput) -> Result<SelfSignedCa> {
    let certificate_der = Pem::try_from(ca_crt_file.get_data())?;
    let signing_key_der = Pem::try_from(ca_key_file.get_data())?;
    let ca = SelfSignedCa::from_der(certificate_der.contents(), signing_key_der.contents())?;

    Ok(ca)
}

pub fn write_key_pair(certificate: &impl AsRef<[u8]>, key: &SigningKey, file_prefix: &str, force: bool) -> Result<()> {
    // Verify certificate and key files do not exist before writing to either
    let crt_file = format!("{}.crt.pem", file_prefix);
    let crt_path = Path::new(&crt_file);
    assert_not_exists(crt_path, force)?;

    let key_file = format!("{}.key.pem", file_prefix);
    let key_path = Path::new(&key_file);
    assert_not_exists(key_path, force)?;

    write_certificate(crt_path, certificate)?;
    write_signing_key(key_path, key)?;

    Ok(())
}

fn assert_not_exists(file_path: &Path, force: bool) -> Result<()> {
    if file_path.exists() && !force {
        return Err(anyhow!("Target file '{}' already exists", file_path.display()));
    }
    Ok(())
}

fn write_certificate(file_path: &Path, certificate: &impl AsRef<[u8]>) -> Result<()> {
    let crt_pem = Pem::new("CERTIFICATE", certificate.as_ref());
    fs::write(
        file_path,
        pem::encode_config(&crt_pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )?;
    eprintln!("Certificate stored in '{}'", file_path.display());
    Ok(())
}

fn write_signing_key(file_path: &Path, key: &SigningKey) -> Result<()> {
    let key_pkcs8_der = key.to_pkcs8_der()?;
    let key_pem = Pem::new("PRIVATE KEY", key_pkcs8_der.as_bytes());
    fs::write(
        file_path,
        pem::encode_config(&key_pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )?;
    eprintln!("Key stored in '{}'", file_path.display());
    Ok(())
}
