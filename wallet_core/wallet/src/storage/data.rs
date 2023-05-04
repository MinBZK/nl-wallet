use serde::{Deserialize, Serialize};
use wallet_common::account::{serialization::Base64Bytes, WalletCertificate};

pub trait Keyed {
    const KEY: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
}

impl Keyed for Registration {
    const KEY: &'static str = "registration";
}
