use chrono::DateTime;
use chrono::Utc;
use derive_more::Debug;
use p256::ecdsa::VerifyingKey;
use semver::Version;
use serde::Serialize;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;
use wallet_account::messages::transfer::TransferSessionState;

pub type WalletId = String;

#[derive(Debug)]
pub struct WalletUser {
    pub id: Uuid,
    pub wallet_id: WalletId,
    pub hw_pubkey: VerifyingKey,
    #[debug(skip)]
    pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    #[debug(skip)]
    pub encrypted_previous_pin_pubkey: Option<Encrypted<VerifyingKey>>,
    pub unsuccessful_pin_entries: u8,
    pub last_unsuccessful_pin_entry: Option<DateTime<Utc>>,
    pub instruction_challenge: Option<InstructionChallenge>,
    pub instruction_sequence_number: u64,
    pub attestation: WalletUserAttestation,
    pub state: WalletUserState,
    pub recovery_code: Option<String>,
    pub transfer_session: Option<TransferSession>,
}

#[derive(Debug, Clone)]
pub struct TransferSession {
    pub id: Uuid,
    pub destination_wallet_user_id: Uuid,
    pub destination_wallet_recovery_code: String,
    pub transfer_session_id: Uuid,
    pub destination_wallet_app_version: Version,
    pub state: TransferSessionState,
    pub encrypted_wallet_data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum WalletUserAttestation {
    Apple { assertion_counter: AssertionCounter },
    Android,
}

impl WalletUser {
    pub fn pin_change_in_progress(&self) -> bool {
        self.encrypted_previous_pin_pubkey.is_some()
    }

    pub fn transfer_in_progress(&self) -> bool {
        self.transfer_session
            .as_ref()
            .map(|s| {
                [
                    TransferSessionState::ReadyForTransfer,
                    TransferSessionState::ReadyForDownload,
                ]
                .contains(&s.state)
            })
            .unwrap_or(false)
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct InstructionChallenge {
    pub bytes: Vec<u8>,
    pub expiration_date_time: DateTime<Utc>,
}

#[derive(Debug)]
pub enum WalletUserQueryResult {
    Found(Box<WalletUser>),
    NotFound,
}

#[derive(Debug)]
pub struct WalletUserCreate {
    pub wallet_id: String,
    pub hw_pubkey: VerifyingKey,
    #[debug(skip)]
    pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    pub attestation_date_time: DateTime<Utc>,
    pub attestation: WalletUserAttestationCreate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum WalletUserState {
    Active,
    Blocked,
    RecoveringPin,
}

#[derive(Debug)]
pub enum WalletUserAttestationCreate {
    Apple {
        data: Vec<u8>,
        assertion_counter: AssertionCounter,
    },
    Android {
        certificate_chain: Vec<Vec<u8>>,
        integrity_verdict_json: String,
    },
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

    use crypto::utils::random_bytes;
    use hsm::model::encrypted::Encrypted;
    use hsm::model::encrypted::InitializationVector;

    use crate::model::wallet_user::WalletUser;
    use crate::model::wallet_user::WalletUserAttestation;
    use crate::model::wallet_user::WalletUserState;

    pub fn wallet_user_1() -> WalletUser {
        WalletUser {
            id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
            wallet_id: "wallet_123".to_string(),
            hw_pubkey: VerifyingKey::from_str(
                r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEhaPRcKTAS30m0409bpOzQLfLNOh5
SssTb0eI53lvfdvG/xkNcktwsXEIPL1y3lUKn1u1ZhFTnQn4QKmnvaN4uQ==
-----END PUBLIC KEY-----
"#,
            )
            .unwrap(),
            encrypted_pin_pubkey: Encrypted::new(random_bytes(32), InitializationVector(random_bytes(32))),
            encrypted_previous_pin_pubkey: None,
            unsuccessful_pin_entries: 0,
            last_unsuccessful_pin_entry: None,
            instruction_challenge: None,
            instruction_sequence_number: 0,
            attestation: WalletUserAttestation::Android,
            state: WalletUserState::Active,
            recovery_code: None,
            transfer_session: None,
        }
    }
}
