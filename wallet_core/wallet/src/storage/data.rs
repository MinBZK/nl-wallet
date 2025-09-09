use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::base64::Base64;
use serde_with::serde_as;

use wallet_account::messages::registration::WalletCertificate;

use crate::pin::change::State;
use crate::transfer::TransferSessionId;

pub trait KeyedData: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyData {
    pub identifier: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub attested_key_identifier: String,
    #[serde_as(as = "Base64")]
    pub pin_salt: Vec<u8>,
    pub wallet_id: String,
    pub wallet_certificate: WalletCertificate,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct InstructionData {
    pub instruction_sequence_number: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, strum::Display)]
#[serde(rename_all_fields = "snake_case")]
pub enum UnlockMethod {
    #[default]
    PinCode,
    PinCodeAndBiometrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePinData {
    pub state: State,
}

impl UnlockMethod {
    pub fn has_biometrics(&self) -> bool {
        match self {
            Self::PinCode => false,
            Self::PinCodeAndBiometrics => true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnlockData {
    pub method: UnlockMethod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransferData {
    pub transfer_session_id: TransferSessionId,
}

impl KeyedData for KeyData {
    const KEY: &'static str = "key";
}

impl KeyedData for RegistrationData {
    const KEY: &'static str = "registration";
}

impl KeyedData for InstructionData {
    const KEY: &'static str = "instructions";
}

impl KeyedData for UnlockData {
    const KEY: &'static str = "unlock";
}

impl KeyedData for ChangePinData {
    const KEY: &'static str = "change_pin";
}

impl KeyedData for TransferData {
    const KEY: &'static str = "transfer_data";
}
