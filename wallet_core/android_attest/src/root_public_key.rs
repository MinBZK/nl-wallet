use std::sync::LazyLock;

use const_decoder::Pem;
use p256::ecdsa::VerifyingKey;
use rsa::RsaPublicKey;
use spki::DecodePublicKey;

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

#[derive(Debug, Clone)]
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
