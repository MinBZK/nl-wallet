use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use wallet_common::account::messages::auth::WalletCertificate;

pub trait KeyedData: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    #[serde_as(as = "Base64")]
    pub pin_salt: Vec<u8>,
    pub wallet_certificate: WalletCertificate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstructionData {
    pub instruction_sequence_number: u64,
}

impl KeyedData for RegistrationData {
    const KEY: &'static str = "registration";
}

impl KeyedData for InstructionData {
    const KEY: &'static str = "instructions";
}
