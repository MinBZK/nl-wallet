use std::fs;
use std::io;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use clio::CachedInput;
use p256::ecdsa::SigningKey;
use p256::pkcs8::DecodePrivateKey;
use p256::pkcs8::EncodePrivateKey;
use pem::EncodeConfig;
use pem::LineEnding;
use pem::Pem;

use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;

fn read_certificate(input: CachedInput) -> Result<BorrowingCertificate> {
    let input_string = io::read_to_string(input)?;
    let crt = BorrowingCertificate::from_pem(input_string.as_bytes())?;
    Ok(crt)
}

fn read_signing_key(input: CachedInput) -> Result<SigningKey> {
    let pem: Pem = io::read_to_string(input)?.parse()?;
    let key = SigningKey::from_pkcs8_der(pem.contents())?;
    Ok(key)
}

pub fn read_key_pair(ca_key_file: CachedInput, ca_crt_file: CachedInput) -> Result<KeyPair> {
    let ca_crt = read_certificate(ca_crt_file)?;
    let ca_key = read_signing_key(ca_key_file)?;
    let key_pair = KeyPair::new_from_signing_key(ca_key, ca_crt)?;
    Ok(key_pair)
}

pub fn write_key_pair(key_pair: &KeyPair, file_prefix: &str, force: bool) -> Result<()> {
    // Verify certificate and key files do not exist before writing to either
    let crt_file = format!("{}.crt.pem", file_prefix);
    let crt_path = Path::new(&crt_file);
    assert_not_exists(crt_path, force)?;

    let key_file = format!("{}.key.pem", file_prefix);
    let key_path = Path::new(&key_file);
    assert_not_exists(key_path, force)?;

    write_certificate(crt_path, key_pair.certificate())?;
    write_signing_key(key_path, key_pair.private_key())?;

    Ok(())
}

fn assert_not_exists(file_path: &Path, force: bool) -> Result<()> {
    if file_path.exists() && !force {
        return Err(anyhow!("Target file '{}' already exists", file_path.display()));
    }
    Ok(())
}

fn write_certificate(file_path: &Path, certificate: &BorrowingCertificate) -> Result<()> {
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
