use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct WalletUser {
    pub id: uuid::Uuid,
    pub wallet_id: String,
}

#[derive(Deserialize)]
pub struct WalletUserCreate {
    pub id: uuid::Uuid,
    pub wallet_id: String,
    pub hw_pubkey: String,
}
