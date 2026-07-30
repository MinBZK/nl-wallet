use std::fs;
use std::mem::size_of;
use std::path::Path;

use anyhow::Result;
use anyhow::anyhow;
use clio::CachedInput;
use crypto::server_keys::generate::Ca;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePrivateKey;
use pem::EncodeConfig;
use pem::LineEnding;
use pem::Pem;
use rcgen::CertificateRevocationList;
use time::OffsetDateTime;
use x509_parser::parse_x509_crl;

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

pub fn write_crl(file_prefix: &str, crl: &CertificateRevocationList, force: bool) -> Result<()> {
    let pem_file = format!("{file_prefix}.crl.pem");
    let pem_path = Path::new(&pem_file);
    let der_file = format!("{file_prefix}.crl.der");
    let der_path = Path::new(&der_file);
    assert_not_exists(pem_path, force)?;
    assert_not_exists(der_path, force)?;

    fs::write(pem_path, crl.pem()?)?;
    fs::write(der_path, crl.der())?;
    eprintln!("CRL stored in '{}'", pem_path.display());
    eprintln!("CRL stored in '{}'", der_path.display());
    Ok(())
}

fn select_crl_number(timestamp_number: u64, previous_number: u64) -> Option<u64> {
    previous_number
        .checked_add(1)
        .map(|incremented_number| timestamp_number.max(incremented_number))
}

/// Select a `crlNumber` for an output prefix. A new sequence starts at `this_update`'s Unix timestamp. When a DER CRL
/// already exists, the result is guaranteed to exceed its number even if the clock has not advanced or moved backwards.
pub fn next_crl_number(file_prefix: &str, this_update: OffsetDateTime) -> Result<u64> {
    let timestamp_number = u64::try_from(this_update.unix_timestamp())
        .map_err(|_| anyhow!("CRL thisUpdate must be on or after the Unix epoch"))?;
    let der_file = format!("{file_prefix}.crl.der");
    let der_path = Path::new(&der_file);
    if !der_path.exists() {
        return Ok(timestamp_number);
    }

    let der = fs::read(der_path)?;
    let (remainder, previous_crl) = parse_x509_crl(&der)
        .map_err(|error| anyhow!("Could not parse existing CRL '{}': {error}", der_path.display()))?;
    if !remainder.is_empty() {
        return Err(anyhow!("Existing CRL '{}' contains trailing data", der_path.display()));
    }
    let previous_number = previous_crl
        .crl_number()
        .ok_or_else(|| anyhow!("Existing CRL '{}' has no crlNumber", der_path.display()))?;
    let previous_number_bytes = previous_number.to_bytes_be();
    if previous_number_bytes.len() > size_of::<u64>() {
        return Err(anyhow!(
            "Existing CRL '{}' has a crlNumber that is too large",
            der_path.display()
        ));
    }
    let previous_number = previous_number_bytes
        .into_iter()
        .fold(0_u64, |value, byte| (value << 8) | u64::from(byte));
    select_crl_number(timestamp_number, previous_number).ok_or_else(|| {
        anyhow!(
            "Existing CRL '{}' has the maximum supported crlNumber",
            der_path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::select_crl_number;

    #[test]
    fn crl_number_increases_when_clock_does_not() {
        assert_eq!(select_crl_number(42, 42), Some(43));
        assert_eq!(select_crl_number(41, 42), Some(43));
    }

    #[test]
    fn crl_number_uses_later_timestamp_and_rejects_overflow() {
        assert_eq!(select_crl_number(44, 42), Some(44));
        assert_eq!(select_crl_number(42, u64::MAX), None);
    }
}
