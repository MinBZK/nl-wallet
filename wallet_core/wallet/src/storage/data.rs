use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wallet_common::account::{serialization::Base64Bytes, WalletCertificate};

pub trait KeyedData: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    const KEY: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
}

impl KeyedData for Registration {
    const KEY: &'static str = "registration";
}
