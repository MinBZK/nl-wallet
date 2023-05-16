use serde::{de::DeserializeOwned, Deserialize, Serialize};

use wallet_common::account::{auth::WalletCertificate, serialization::Base64Bytes};

pub trait KeyedData: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    const KEY: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
}

impl KeyedData for RegistrationData {
    const KEY: &'static str = "registration";
}
