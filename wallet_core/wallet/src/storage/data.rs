use serde::{de::DeserializeOwned, Deserialize, Serialize};

use wallet_common::account::{messages::auth::WalletCertificate, serialization::Base64Bytes};

pub trait KeyedData: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    const KEY: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
    pub instruction_sequence_number: u64,
}

impl KeyedData for RegistrationData {
    const KEY: &'static str = "registration";
}
