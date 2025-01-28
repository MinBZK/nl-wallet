use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use uuid;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::database::Db;
use crate::transaction;
use crate::transaction::Transaction;
use crate::wallet_user;
use crate::wallet_user_key;

pub struct Repositories(Db);

impl Repositories {
    pub fn new(db: Db) -> Self {
        Self(db)
    }
}

impl TransactionStarter for Repositories {
    type TransactionType = Transaction;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
        transaction::begin_transaction(&self.0).await
    }
}

impl WalletUserRepository for Repositories {
    type TransactionType = Transaction;

    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<Uuid, PersistenceError> {
        wallet_user::create_wallet_user(transaction, user).await
    }

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult, PersistenceError> {
        wallet_user::find_wallet_user_by_wallet_id(transaction, wallet_id).await
    }

    async fn clear_instruction_challenge(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::clear_instruction_challenge(transaction, wallet_id).await
    }

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number).await
    }

    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        challenge: InstructionChallenge,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_challenge_and_sequence_number(
            transaction,
            wallet_id,
            challenge,
            instruction_sequence_number,
        )
        .await
    }

    async fn register_unsuccessful_pin_entry(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        is_blocked: bool,
        datetime: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        wallet_user::register_unsuccessful_pin_entry(transaction, wallet_id, is_blocked, datetime).await
    }

    async fn reset_unsuccessful_pin_entries(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::reset_unsuccessful_pin_entries(transaction, wallet_id).await
    }

    async fn save_keys(
        &self,
        transaction: &Self::TransactionType,
        keys: WalletUserKeys,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::create_keys(transaction, keys).await
    }

    async fn find_keys_by_identifiers(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key_identifiers: &[String],
    ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
        wallet_user_key::find_keys_by_identifiers(transaction, wallet_user_id, key_identifiers).await
    }

    async fn change_pin(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        new_encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    ) -> Result<(), PersistenceError> {
        wallet_user::change_pin(transaction, wallet_id, new_encrypted_pin_pubkey).await
    }

    async fn commit_pin_change(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::commit_pin_change(transaction, wallet_id).await
    }

    async fn rollback_pin_change(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::rollback_pin_change(transaction, wallet_id).await
    }

    async fn save_wte_issued(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::save_wte_issued(transaction, wallet_id).await
    }

    async fn update_apple_assertion_counter(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        assertion_counter: AssertionCounter,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_apple_assertion_counter(transaction, wallet_id, assertion_counter).await
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::collections::HashMap;
    use std::time::Duration;

    use chrono::DateTime;
    use chrono::Utc;
    use mockall;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;
    use uuid::uuid;
    use uuid::Uuid;

    use apple_app_attest::AssertionCounter;
    use hsm::model::encrypted::Encrypted;
    use hsm::model::wrapped_key::WrappedKey;
    use wallet_common::account::serialization::DerVerifyingKey;
    use wallet_provider_domain::model::wallet_user::InstructionChallenge;
    use wallet_provider_domain::model::wallet_user::WalletUser;
    use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
    use wallet_provider_domain::model::wallet_user::WalletUserCreate;
    use wallet_provider_domain::model::wallet_user::WalletUserKeys;
    use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_domain::repository::MockTransactionStarter;
    use wallet_provider_domain::repository::PersistenceError;
    use wallet_provider_domain::repository::TransactionStarter;
    use wallet_provider_domain::repository::WalletUserRepository;

    mockall::mock! {
        pub TransactionalWalletUserRepository {}

        impl WalletUserRepository for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn create_wallet_user(
                &self,
                transaction: &MockTransaction,
                user: WalletUserCreate,
            ) -> Result<Uuid, PersistenceError>;

            async fn find_wallet_user_by_wallet_id(
                &self,
                _transaction: &MockTransaction,
                wallet_id: &str,
            ) -> Result<WalletUserQueryResult, PersistenceError>;

            async fn register_unsuccessful_pin_entry(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _is_blocked: bool,
                _datetime: DateTime<Utc>,
            ) -> Result<(), PersistenceError>;

            async fn reset_unsuccessful_pin_entries(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
            ) -> Result<(), PersistenceError>;

            async fn clear_instruction_challenge(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_challenge_and_sequence_number(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _challenge: InstructionChallenge,
                _instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_sequence_number(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn save_keys(
                &self,
                _transaction: &MockTransaction,
                _keys: WalletUserKeys,
            ) -> Result<(), PersistenceError>;

            async fn find_keys_by_identifiers(
                &self,
                _transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key_identifiers: &[String],
            ) -> Result<HashMap<String, WrappedKey>, PersistenceError>;

            async fn change_pin(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str,
                encrypted_pin_pubkey: Encrypted<VerifyingKey>,
            ) -> Result<(), PersistenceError>;

            async fn commit_pin_change(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str
            ) -> Result<(), PersistenceError>;

            async fn rollback_pin_change(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str
            ) -> Result<(), PersistenceError>;

            async fn save_wte_issued(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str
            ) -> Result<(), PersistenceError>;

            async fn update_apple_assertion_counter(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str,
                assertion_counter: AssertionCounter,
            ) -> Result<(), PersistenceError>;
        }

        impl TransactionStarter for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn begin_transaction(&self) -> Result<MockTransaction, PersistenceError>;
        }
    }

    #[derive(Clone)]
    pub struct WalletUserTestRepo {
        pub hw_pubkey: VerifyingKey,
        pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        pub previous_encrypted_pin_pubkey: Option<Encrypted<VerifyingKey>>,
        pub challenge: Option<Vec<u8>>,
        pub instruction_sequence_number: u64,
        pub apple_assertion_counter: Option<AssertionCounter>,
    }

    impl WalletUserRepository for WalletUserTestRepo {
        type TransactionType = MockTransaction;

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<Uuid, PersistenceError> {
            Ok(uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"))
        }

        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            wallet_id: &str,
        ) -> Result<WalletUserQueryResult, PersistenceError> {
            Ok(WalletUserQueryResult::Found(Box::new(WalletUser {
                id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                wallet_id: wallet_id.to_string(),
                hw_pubkey: DerVerifyingKey(self.hw_pubkey),
                encrypted_pin_pubkey: self.encrypted_pin_pubkey.clone(),
                encrypted_previous_pin_pubkey: self.previous_encrypted_pin_pubkey.clone(),
                unsuccessful_pin_entries: 0,
                last_unsuccessful_pin_entry: None,
                instruction_challenge: self.challenge.clone().map(|c| InstructionChallenge {
                    bytes: c,
                    expiration_date_time: Utc::now() + Duration::from_millis(15000),
                }),
                instruction_sequence_number: self.instruction_sequence_number,
                has_wte: false,
                attestation: match self.apple_assertion_counter {
                    Some(assertion_counter) => WalletUserAttestation::Apple { assertion_counter },
                    None => WalletUserAttestation::Android,
                },
            })))
        }

        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _is_blocked: bool,
            _datetime: DateTime<Utc>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn reset_unsuccessful_pin_entries(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn update_instruction_challenge_and_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _challenge: InstructionChallenge,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn update_instruction_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn save_keys(
            &self,
            _transaction: &Self::TransactionType,
            _keys: WalletUserKeys,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn find_keys_by_identifiers(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            key_identifiers: &[String],
        ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
            Ok(key_identifiers
                .iter()
                .map(|id| {
                    let privkey = SigningKey::random(&mut OsRng);

                    (
                        id.clone(),
                        WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
                    )
                })
                .collect())
        }

        async fn change_pin(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn commit_pin_change(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn rollback_pin_change(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn save_wte_issued(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn update_apple_assertion_counter(
            &self,
            _transaction: &MockTransaction,
            _wallet_id: &str,
            _assertion_counter: AssertionCounter,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    impl TransactionStarter for WalletUserTestRepo {
        type TransactionType = <MockTransactionStarter as TransactionStarter>::TransactionType;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            MockTransactionStarter.begin_transaction().await
        }
    }
}
