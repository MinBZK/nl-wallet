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

use crypto::server_keys::generate::Ca;

pub fn read_public_key(public_key_file: &CachedInput) -> Result<Pem> {
    let pem = Pem::try_from(public_key_file.get_data())?;
    assert_eq!(pem.tag(), "PUBLIC KEY");
    Ok(pem)
}

pub fn read_self_signed_ca(ca_crt_file: &CachedInput, ca_key_file: &CachedInput) -> Result<Ca> {
    let certificate_der = Pem::try_from(ca_crt_file.get_data())?;
    let signing_key_der = Pem::try_from(ca_key_file.get_data())?;
    let ca = Ca::from_der(certificate_der.contents(), signing_key_der.contents())?;

    Ok(ca)
}

pub fn write_certificate(certificate: &impl AsRef<[u8]>, file_prefix: &str, force: bool) -> Result<()> {
    // Verify certificate file does not exist before writing (depending on force)
    let crt_file = format!("{file_prefix}.crt.pem");
    let crt_path = Path::new(&crt_file);
    assert_not_exists(crt_path, force)?;

    write_certificate_inner(crt_path, certificate)?;

    Ok(())
}

pub fn write_key_pair(certificate: &impl AsRef<[u8]>, key: &SigningKey, file_prefix: &str, force: bool) -> Result<()> {
    // Verify certificate and key files do not exist before writing to either (depending on force)
    // We verify this before calling write_certificate_inner to avoid writing the certificate if the key file fails.
    let key_file = format!("{file_prefix}.key.pem");
    let key_path = Path::new(&key_file);
    assert_not_exists(key_path, force)?;

    // This verifies the certificate file does not exist before writing the certificate (depending on force)
    write_certificate(certificate, file_prefix, force)?;
    write_signing_key_inner(key_path, key)?;

    Ok(())
}

fn assert_not_exists(file_path: &Path, force: bool) -> Result<()> {
    if file_path.exists() && !force {
        return Err(anyhow!("Target file '{}' already exists", file_path.display()));
    }
    Ok(())
}

fn write_certificate_inner(file_path: &Path, certificate: &impl AsRef<[u8]>) -> Result<()> {
    let crt_pem = Pem::new("CERTIFICATE", certificate.as_ref());
    fs::write(
        file_path,
        pem::encode_config(&crt_pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )?;
    eprintln!("Certificate stored in '{}'", file_path.display());
    Ok(())
}

fn write_signing_key_inner(file_path: &Path, key: &SigningKey) -> Result<()> {
    let key_pkcs8_der = key.to_pkcs8_der()?;
    let key_pem = Pem::new("PRIVATE KEY", key_pkcs8_der.as_bytes());
    fs::write(
        file_path,
        pem::encode_config(&key_pem, EncodeConfig::new().set_line_ending(LineEnding::LF)),
    )?;
    eprintln!("Key stored in '{}'", file_path.display());
    Ok(())
}
