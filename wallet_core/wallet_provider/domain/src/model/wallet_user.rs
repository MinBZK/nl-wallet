use chrono::{DateTime, Local};
use p256::ecdsa::VerifyingKey;
use serde::Serialize;
use uuid::Uuid;

use crate::model::wrapped_key::WrappedKey;
use wallet_common::account::serialization::DerVerifyingKey;

pub type WalletId = String;

#[derive(Serialize, Debug)]
pub struct WalletUser {
    pub id: Uuid,
    pub wallet_id: WalletId,
    pub hw_pubkey: DerVerifyingKey,
    pub pin_pubkey: DerVerifyingKey,
    pub unsuccessful_pin_entries: u8,
    pub last_unsuccessful_pin_entry: Option<DateTime<Local>>,
    pub instruction_challenge: Option<InstructionChallenge>,
    pub instruction_sequence_number: u64,
}

#[derive(Clone, Serialize, Debug)]
pub struct InstructionChallenge {
    pub bytes: Vec<u8>,
    pub expiration_date_time: DateTime<Local>,
}

#[derive(Debug)]
pub enum WalletUserQueryResult {
    Found(Box<WalletUser>),
    NotFound,
    Blocked,
}

pub struct WalletUserCreate {
    pub id: Uuid,
    pub wallet_id: String,
    pub hw_pubkey: VerifyingKey,
    pub pin_pubkey: VerifyingKey,
}

#[derive(Clone)]
pub struct WalletUserKeys {
    pub wallet_user_id: Uuid,
    pub keys: Vec<WalletUserKey>,
}

#[derive(Clone)]
pub struct WalletUserKey {
    pub wallet_user_key_id: Uuid,
    pub key_identifier: String,
    pub key: WrappedKey,
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::str::FromStr;

    use p256::ecdsa::VerifyingKey;
    use uuid::uuid;

    use wallet_common::account::serialization::DerVerifyingKey;

    use crate::model::wallet_user::WalletUser;

    pub fn wallet_user_1() -> WalletUser {
        WalletUser {
            id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
            wallet_id: "wallet_123".to_string(),
            hw_pubkey: DerVerifyingKey(
                VerifyingKey::from_str(
                    r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEhaPRcKTAS30m0409bpOzQLfLNOh5
SssTb0eI53lvfdvG/xkNcktwsXEIPL1y3lUKn1u1ZhFTnQn4QKmnvaN4uQ==
-----END PUBLIC KEY-----
"#,
                )
                .unwrap(),
            ),
            pin_pubkey: DerVerifyingKey(
                VerifyingKey::from_str(
                    r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE5hSrSlRFtqYZ5zP+Fth8wwRGBsk4
3y/LssXgXj1H3QExJGtEtGlh/LqYPvFwdaNvMYgUtpummzqvIgiuIiOYig==
-----END PUBLIC KEY-----
"#,
                )
                .unwrap(),
            ),
            unsuccessful_pin_entries: 0,
            last_unsuccessful_pin_entry: None,
            instruction_challenge: None,
            instruction_sequence_number: 0,
        }
    }
}
