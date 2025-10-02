use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use derive_more::From;
use p256::ecdsa::VerifyingKey;
use semver::Version;
use uuid;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;
use wallet_account::messages::transfer::TransferSessionState;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::TransferSession;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::model::wallet_user::WalletUserPinRecoveryKeys;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::database::Db;
use crate::transaction;
use crate::transaction::Transaction;
use crate::wallet_user;
use crate::wallet_user_key;

#[derive(From)]
pub struct Repositories(Db);

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

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number).await
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

    async fn save_pin_recovery_keys(
        &self,
        transaction: &Self::TransactionType,
        keys: WalletUserPinRecoveryKeys,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::create_pin_recovery_keys(transaction, keys).await
    }

    async fn is_pin_recovery_key(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        key: VerifyingKey,
    ) -> Result<bool, PersistenceError> {
        wallet_user_key::is_pin_recovery_key(transaction, wallet_id, key).await
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
        user_state: WalletUserState,
    ) -> Result<(), PersistenceError> {
        wallet_user::change_pin(transaction, wallet_id, new_encrypted_pin_pubkey, user_state).await
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

    async fn store_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        recovery_code: String,
    ) -> Result<(), PersistenceError> {
        wallet_user::store_recovery_code(transaction, wallet_id, recovery_code).await
    }

    async fn recover_pin(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<(), PersistenceError> {
        wallet_user::recover_pin(transaction, wallet_id).await?;
        wallet_user_key::delete_pin_recovery_keys(transaction, wallet_id).await
    }

    async fn has_multiple_active_accounts_by_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: &str,
    ) -> Result<bool, PersistenceError> {
        wallet_user::has_multiple_active_accounts_by_recovery_code(transaction, recovery_code).await
    }

    async fn update_apple_assertion_counter(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        assertion_counter: AssertionCounter,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_apple_assertion_counter(transaction, wallet_id, assertion_counter).await
    }

    async fn create_transfer_session(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
        destination_wallet_app_version: Version,
        created: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        wallet_user::create_transfer_session(
            transaction,
            destination_wallet_user_id,
            transfer_session_id,
            destination_wallet_app_version,
            created,
        )
        .await
    }

    async fn find_transfer_session_by_transfer_session_id(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
    ) -> Result<Option<TransferSession>, PersistenceError> {
        wallet_user::find_transfer_session_by_transfer_session_id(transaction, transfer_session_id).await
    }

    async fn find_transfer_session_id_by_destination_wallet_user_id(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
    ) -> Result<Option<Uuid>, PersistenceError> {
        wallet_user::find_transfer_session_id_by_destination_wallet_user_id(transaction, destination_wallet_user_id)
            .await
    }

    async fn confirm_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_transfer_state(transaction, transfer_session_id, TransferSessionState::ReadyForTransfer)
            .await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            source_wallet_user_id,
            WalletUserState::Active,
            WalletUserState::Transferring,
        )
        .await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            destination_wallet_user_id,
            WalletUserState::Active,
            WalletUserState::Transferring,
        )
        .await
    }

    async fn cancel_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Option<Uuid>,
        destination_wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        if let Some(wallet_user_id) = source_wallet_user_id {
            wallet_user::transition_wallet_user_state(
                transaction,
                wallet_user_id,
                WalletUserState::Transferring,
                WalletUserState::Active,
            )
            .await?;
        }
        wallet_user::transition_wallet_user_state(
            transaction,
            destination_wallet_user_id,
            WalletUserState::Transferring,
            WalletUserState::Active,
        )
        .await?;
        wallet_user::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Canceled).await?;
        wallet_user::set_wallet_transfer_data(transaction, transfer_session_id, None).await
    }

    async fn store_wallet_transfer_data(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        encrypted_wallet_data: String,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_transfer_state(transaction, transfer_session_id, TransferSessionState::ReadyForDownload)
            .await?;
        wallet_user::set_wallet_transfer_data(transaction, transfer_session_id, Some(encrypted_wallet_data)).await
    }

    async fn complete_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::move_keys(transaction, source_wallet_user_id, destination_wallet_user_id).await?;
        wallet_user::set_wallet_transfer_data(transaction, transfer_session_id, None).await?;
        wallet_user::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Success).await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            source_wallet_user_id,
            WalletUserState::Transferring,
            WalletUserState::Transferred,
        )
        .await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            destination_wallet_user_id,
            WalletUserState::Transferring,
            WalletUserState::Active,
        )
        .await?;

        Ok(())
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
    use semver::Version;
    use uuid::Uuid;
    use uuid::uuid;

    use apple_app_attest::AssertionCounter;
    use hsm::model::encrypted::Encrypted;
    use hsm::model::wrapped_key::WrappedKey;
    use wallet_provider_domain::model::wallet_user::InstructionChallenge;
    use wallet_provider_domain::model::wallet_user::TransferSession;
    use wallet_provider_domain::model::wallet_user::WalletUser;
    use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
    use wallet_provider_domain::model::wallet_user::WalletUserCreate;
    use wallet_provider_domain::model::wallet_user::WalletUserKeys;
    use wallet_provider_domain::model::wallet_user::WalletUserPinRecoveryKeys;
    use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
    use wallet_provider_domain::model::wallet_user::WalletUserState;
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

            async fn save_pin_recovery_keys(
                &self,
                _transaction: &MockTransaction,
                _keys: WalletUserPinRecoveryKeys,
            ) -> Result<(), PersistenceError>;

            async fn is_pin_recovery_key(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _key: VerifyingKey,
            ) -> Result<bool, PersistenceError>;

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
                user_state: WalletUserState,
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

            async fn update_apple_assertion_counter(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str,
                assertion_counter: AssertionCounter,
            ) -> Result<(), PersistenceError>;

            async fn store_recovery_code(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str,
                recovery_code: String,
            ) -> Result<(), PersistenceError>;

            async fn recover_pin(
                &self,
                transaction: &MockTransaction,
                wallet_id: &str,
            ) -> Result<(), PersistenceError>;

            async fn has_multiple_active_accounts_by_recovery_code(
                &self,
                transaction: &MockTransaction,
                recovery_code: &str,
            ) -> Result<bool, PersistenceError>;

            async fn create_transfer_session(
                &self,
                transaction: &MockTransaction,
                destination_wallet_user_id: Uuid,
                transfer_session_id: Uuid,
                destination_wallet_app_version: Version,
                created: DateTime<Utc>,
            ) -> Result<(), PersistenceError>;

            async fn find_transfer_session_by_transfer_session_id(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
            ) -> Result<Option<TransferSession>, PersistenceError>;

            async fn find_transfer_session_id_by_destination_wallet_user_id(
                &self,
                transaction: &MockTransaction,
                destination_wallet_user_id: Uuid,
            ) -> Result<Option<Uuid>, PersistenceError>;

            async fn confirm_wallet_transfer(&self,
                transaction: &MockTransaction,
                source_wallet_user_id: Uuid,
                destination_wallet_user_id: Uuid,
                transfer_session_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn cancel_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                source_wallet_user_id: Option<Uuid>,
                destination_wallet_user_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn store_wallet_transfer_data(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                encrypted_wallet_data: String,
            ) -> Result<(), PersistenceError>;

            async fn complete_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                source_wallet_user_id: Uuid,
                destination_wallet_user_id: Uuid,
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
        pub state: WalletUserState,
        pub transfer_session: Option<TransferSession>,
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
                hw_pubkey: self.hw_pubkey,
                encrypted_pin_pubkey: self.encrypted_pin_pubkey.clone(),
                encrypted_previous_pin_pubkey: self.previous_encrypted_pin_pubkey.clone(),
                unsuccessful_pin_entries: 0,
                last_unsuccessful_pin_entry: None,
                instruction_challenge: self.challenge.clone().map(|c| InstructionChallenge {
                    bytes: c,
                    expiration_date_time: Utc::now() + Duration::from_millis(15000),
                }),
                instruction_sequence_number: self.instruction_sequence_number,
                attestation: match self.apple_assertion_counter {
                    Some(assertion_counter) => WalletUserAttestation::Apple { assertion_counter },
                    None => WalletUserAttestation::Android,
                },
                state: self.state,
                recovery_code: None,
            })))
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

        async fn save_keys(
            &self,
            _transaction: &Self::TransactionType,
            _keys: WalletUserKeys,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn save_pin_recovery_keys(
            &self,
            _transaction: &MockTransaction,
            _keys: WalletUserPinRecoveryKeys,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn is_pin_recovery_key(
            &self,
            _transaction: &MockTransaction,
            _wallet_id: &str,
            _key: VerifyingKey,
        ) -> Result<bool, PersistenceError> {
            Ok(true)
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
            _user_state: WalletUserState,
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

        async fn store_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _recovery_code: String,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn recover_pin(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn has_multiple_active_accounts_by_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &str,
        ) -> Result<bool, PersistenceError> {
            Ok(false)
        }

        async fn update_apple_assertion_counter(
            &self,
            _transaction: &MockTransaction,
            _wallet_id: &str,
            _assertion_counter: AssertionCounter,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn create_transfer_session(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
            _destination_wallet_app_version: Version,
            _created: DateTime<Utc>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn find_transfer_session_by_transfer_session_id(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
        ) -> Result<Option<TransferSession>, PersistenceError> {
            Ok(None)
        }

        async fn find_transfer_session_id_by_destination_wallet_user_id(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
        ) -> Result<Option<Uuid>, PersistenceError> {
            Ok(None)
        }

        async fn confirm_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn cancel_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Option<Uuid>,
            _destination_wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn store_wallet_transfer_data(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _encrypted_wallet_data: String,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn complete_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
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
