use serde::{Deserialize, Serialize};
use wallet_common::account::{serialization::Base64Bytes, WalletCertificate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
}
