use serde::{Deserialize, Serialize};
use wallet_common::account::WalletCertificate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub pin_salt: Vec<u8>,
    pub wallet_certificate: WalletCertificate,
}
