use async_trait::async_trait;
use chrono::{DateTime, Local};
use p256::ecdsa::SigningKey;

use crate::model::wallet_user::{WalletUserCreate, WalletUserQueryResult};

use super::{errors::PersistenceError, transaction::Committable};

type Result<T> = std::result::Result<T, PersistenceError>;

#[async_trait]
pub trait WalletUserRepository {
    type TransactionType: Committable;

    async fn create_wallet_user(&self, transaction: &Self::TransactionType, user: WalletUserCreate) -> Result<()>;

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult>;

    async fn clear_instruction_challenge(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        challenge: Option<Vec<u8>>,
        instruction_sequence_number: u64,
    ) -> Result<()>;

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<()>;

    async fn register_unsuccessful_pin_entry(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        is_blocked: bool,
        datetime: DateTime<Local>,
    ) -> Result<()>;

    async fn reset_unsuccessful_pin_entries(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn save_key(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        keys: &[(String, SigningKey)],
    ) -> Result<()>;

    async fn get_key(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        key_identifier: &str,
    ) -> Result<Option<SigningKey>>;

    async fn get_keys<T: AsRef<str> + Sync>(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        key_identifiers: &[T],
    ) -> Result<Vec<Option<SigningKey>>>;
}

#[cfg(feature = "stub")]
pub mod stub {
    use std::str::FromStr;

    use p256::ecdsa::{SigningKey, VerifyingKey};
    use uuid::uuid;

    use wallet_common::account::serialization::DerVerifyingKey;

    use crate::model::wallet_user::WalletUser;

    use super::{super::transaction::stub::TransactionStub, *};

    pub struct WalletUserRepositoryStub;

    #[async_trait]
    impl WalletUserRepository for WalletUserRepositoryStub {
        type TransactionType = TransactionStub;

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<()> {
            Ok(())
        }

        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<WalletUserQueryResult> {
            Ok(WalletUserQueryResult::Found(Box::new(WalletUser {
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
            })))
        }

        async fn update_instruction_challenge_and_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _challenge: Option<Vec<u8>>,
            _instruction_sequence_number: u64,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_instruction_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _instruction_sequence_number: u64,
        ) -> Result<()> {
            Ok(())
        }

        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<()> {
            Ok(())
        }

        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _is_blocked: bool,
            _datetime: DateTime<Local>,
        ) -> Result<()> {
            Ok(())
        }

        async fn reset_unsuccessful_pin_entries(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<()> {
            Ok(())
        }

        async fn save_key(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _keys: &[(String, SigningKey)],
        ) -> Result<()> {
            Ok(())
        }

        async fn get_key(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _key_identifier: &str,
        ) -> Result<Option<SigningKey>> {
            todo!()
        }

        async fn get_keys<T: AsRef<str> + Sync>(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _key_identifiers: &[T],
        ) -> Result<Vec<Option<SigningKey>>> {
            Ok(vec![])
        }
    }
}
