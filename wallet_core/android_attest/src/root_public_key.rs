use std::sync::LazyLock;

use const_decoder::Pem;
use p256::ecdsa::VerifyingKey;
use rsa::traits::PublicKeyParts;
use rsa::BigUint;
use rsa::RsaPublicKey;
use spki::DecodePublicKey;
use x509_parser::public_key::PublicKey;

const GOOGLE_ROOT_PUBKEY_DER: [u8; 550] =
    Pem::decode(include_bytes!("../assets/google_hardware_attestation_root_pubkey.pem"));

const EMULATOR_ROOT_RSA_PUBKEY_DER: [u8; 162] =
    Pem::decode(include_bytes!("../assets/android_emulator_rsa_root_pubkey.pem"));
const EMULATOR_ROOT_ECDSA_PUBKEY_DER: [u8; 91] =
    Pem::decode(include_bytes!("../assets/android_emulator_ec_root_pubkey.pem"));

pub static GOOGLE_ROOT_PUBKEYS: LazyLock<Vec<RootPublicKey>> =
    LazyLock::new(|| vec![RootPublicKey::rsa_from_der(GOOGLE_ROOT_PUBKEY_DER).unwrap()]);

pub static EMULATOR_PUBKEYS: LazyLock<Vec<RootPublicKey>> = LazyLock::new(|| {
    vec![
        RootPublicKey::rsa_from_der(EMULATOR_ROOT_RSA_PUBKEY_DER).unwrap(),
        RootPublicKey::ecdsa_from_der(EMULATOR_ROOT_ECDSA_PUBKEY_DER).unwrap(),
    ]
});

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(derive_more::From))]
pub enum RootPublicKey {
    Rsa(RsaPublicKey),
    Ecdsa(VerifyingKey),
}

impl RootPublicKey {
    pub fn rsa_from_der(der: impl AsRef<[u8]>) -> Result<Self, spki::Error> {
        RsaPublicKey::from_public_key_der(der.as_ref()).map(Self::Rsa)
    }

    pub fn ecdsa_from_der(der: impl AsRef<[u8]>) -> Result<Self, spki::Error> {
        VerifyingKey::from_public_key_der(der.as_ref()).map(Self::Ecdsa)
    }
}

impl<'a> PartialEq<PublicKey<'a>> for RootPublicKey {
    fn eq(&self, other: &PublicKey<'a>) -> bool {
        match (self, other) {
            (Self::Rsa(public_key), PublicKey::RSA(other_key)) => {
                let modulus = BigUint::from_bytes_be(other_key.modulus);
                let exponent = BigUint::from_bytes_be(other_key.exponent);

                *public_key.n() == modulus && *public_key.e() == exponent
            }
            (Self::Ecdsa(verifying_key), PublicKey::EC(other_key)) => {
                verifying_key.to_encoded_point(false).as_bytes() == other_key.data()
            }
            _ => false,
        }
    }
}

impl PartialEq<RootPublicKey> for PublicKey<'_> {
    fn eq(&self, other: &RootPublicKey) -> bool {
        other.eq(self)
    }
}

impl TryFrom<&[u8]> for RootPublicKey {
    type Error = spki::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public_key = RootPublicKey::rsa_from_der(value)
            // Choose to return the RSA parsing error here, as this will most likely be used in production.
            .or_else(|error| RootPublicKey::ecdsa_from_der(value).map_err(|_| error))?;

        Ok(public_key)
    }
}

#[cfg(test)]
mod test {
    use p256::ecdsa::SigningKey;
    use rsa::RsaPrivateKey;
    use rsa::RsaPublicKey;
    use spki::EncodePublicKey;
    use x509_parser::prelude::FromDer;
    use x509_parser::x509::SubjectPublicKeyInfo;

    use super::RootPublicKey;

    #[test]
    fn test_root_public_key_rsa_partial_eq() {
        let mut thread_rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut thread_rng, 512).unwrap();
        let public_key = RootPublicKey::Rsa(RsaPublicKey::from(&private_key));

        let other_private_key = RsaPrivateKey::new(&mut thread_rng, 512).unwrap();
        let other_public_key = RootPublicKey::Rsa(RsaPublicKey::from(&other_private_key));

        let public_key_der = RsaPublicKey::from(&private_key).to_public_key_der().unwrap();
        let (_, x509_public_key_info) = SubjectPublicKeyInfo::from_der(public_key_der.as_bytes()).unwrap();
        let x509_public_key = x509_public_key_info.parsed().unwrap();

        assert_eq!(public_key, x509_public_key);
        assert_eq!(x509_public_key, public_key);

        assert_ne!(other_public_key, x509_public_key);
        assert_ne!(x509_public_key, other_public_key);
    }

    #[test]
    fn test_root_public_key_ecdsa_partial_eq() {
        let mut thread_rng = rand::thread_rng();

        let signing_key = SigningKey::random(&mut thread_rng);
        let verifying_key = RootPublicKey::Ecdsa(*signing_key.verifying_key());

        let other_signing_key = SigningKey::random(&mut thread_rng);
        let other_verifying_key = RootPublicKey::Ecdsa(*other_signing_key.verifying_key());

        let public_key_der = signing_key.verifying_key().to_public_key_der().unwrap();
        let (_, x509_public_key_info) = SubjectPublicKeyInfo::from_der(public_key_der.as_bytes()).unwrap();
        let x509_public_key = x509_public_key_info.parsed().unwrap();

        assert_eq!(verifying_key, x509_public_key);
        assert_eq!(x509_public_key, verifying_key);

        assert_ne!(other_verifying_key, x509_public_key);
        assert_ne!(x509_public_key, other_verifying_key);
    }
}
