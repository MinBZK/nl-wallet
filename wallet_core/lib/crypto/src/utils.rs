use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use ring::error::Unspecified as UnspecifiedRingError;
use ring::hkdf;
use sha2::Digest;
use sha2::Sha256;

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut output = vec![0u8; len];
    rand::thread_rng().fill(&mut output[..]);
    output
}

pub fn random_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}

pub fn sha256(bts: &[u8]) -> Vec<u8> {
    Sha256::digest(bts).to_vec()
}

/// Compute the HKDF from [RFC 5869](https://tools.ietf.org/html/rfc5869).
pub fn hkdf(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Result<Vec<u8>, UnspecifiedRingError> {
    struct HkdfLen(usize);
    impl hkdf::KeyType for HkdfLen {
        fn len(&self) -> usize {
            self.0
        }
    }

    let mut bts = vec![0u8; len];
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);

    salt.extract(input_key_material)
        .expand(&[info.as_bytes()], HkdfLen(len))?
        .fill(bts.as_mut_slice())?;

    Ok(bts)
}
