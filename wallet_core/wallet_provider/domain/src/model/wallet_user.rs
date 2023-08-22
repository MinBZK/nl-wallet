use chrono::{DateTime, Local};
use serde::Serialize;
use wallet_common::account::serialization::DerVerifyingKey;

#[derive(Serialize, Debug)]
pub struct WalletUser {
    pub id: uuid::Uuid,
    pub wallet_id: String,
    pub hw_pubkey: DerVerifyingKey,
    pub pin_pubkey: DerVerifyingKey,
    pub unsuccessful_pin_entries: u8,
    pub last_unsuccessful_pin_entry: Option<DateTime<Local>>,
    pub instruction_challenge: Option<Vec<u8>>,
    pub instruction_sequence_number: u64,
}

pub enum WalletUserQueryResult {
    Found(Box<WalletUser>),
    NotFound,
    Blocked,
}

pub struct WalletUserCreate {
    pub id: uuid::Uuid,
    pub wallet_id: String,
    pub hw_pubkey_der: Vec<u8>,
    pub pin_pubkey_der: Vec<u8>,
}
