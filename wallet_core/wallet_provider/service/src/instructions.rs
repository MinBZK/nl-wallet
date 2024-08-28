use std::sync::Arc;

use futures::future;
use serde::Serialize;
use uuid::Uuid;

use wallet_common::{
    account::{
        messages::instructions::{CheckPin, GenerateKey, GenerateKeyResult, Sign, SignResult},
        serialization::{DerSignature, DerVerifyingKey},
    },
    generator::Generator,
};
use wallet_provider_domain::{
    model::{
        hsm::WalletUserHsm,
        wallet_user::{WalletUser, WalletUserKey, WalletUserKeys},
    },
    repository::{Committable, TransactionStarter, WalletUserRepository},
};

use crate::{account_server::InstructionError, hsm::HsmError};

pub trait HandleInstruction {
    type Result: Serialize;

    async fn handle<T, R>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &impl WalletUserHsm<Error = HsmError>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>;
}

impl HandleInstruction for CheckPin {
    type Result = ();

    async fn handle<T, R>(
        self,
        _wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        _wallet_user_repository: &R,
        _wallet_user_hsm: &impl WalletUserHsm<Error = HsmError>,
    ) -> Result<(), InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    {
        Ok(())
    }
}

impl HandleInstruction for GenerateKey {
    type Result = GenerateKeyResult;

    async fn handle<T, R>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &impl WalletUserHsm<Error = HsmError>,
    ) -> Result<GenerateKeyResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    {
        let identifiers: Vec<&str> = self.identifiers.iter().map(|i| i.as_str()).collect();
        let keys = wallet_user_hsm.generate_wrapped_keys(&identifiers).await?;

        let (public_keys, wrapped_keys): (Vec<(String, DerVerifyingKey)>, Vec<WalletUserKey>) = keys
            .into_iter()
            .map(|(identifier, public_key, wrapped_key)| {
                (
                    (identifier.clone(), DerVerifyingKey::from(public_key)),
                    WalletUserKey {
                        wallet_user_key_id: uuid_generator.generate(),
                        key_identifier: identifier,
                        key: wrapped_key,
                    },
                )
            })
            .unzip();

        let tx = wallet_user_repository.begin_transaction().await?;
        wallet_user_repository
            .save_keys(
                &tx,
                WalletUserKeys {
                    wallet_user_id: wallet_user.id,
                    keys: wrapped_keys,
                },
            )
            .await?;
        tx.commit().await?;

        Ok(GenerateKeyResult { public_keys })
    }
}

impl HandleInstruction for Sign {
    type Result = SignResult;

    async fn handle<T, R>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &impl WalletUserHsm<Error = HsmError>,
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    {
        let (data, identifiers): (Vec<_>, Vec<_>) = self.messages_with_identifiers.into_iter().unzip();

        let tx = wallet_user_repository.begin_transaction().await?;
        let found_keys = wallet_user_repository
            .find_keys_by_identifiers(
                &tx,
                wallet_user.id,
                &identifiers.clone().into_iter().flatten().collect::<Vec<_>>(),
            )
            .await?;
        tx.commit().await?;

        let signatures = future::try_join_all(identifiers.iter().zip(data).map(|(identifiers, data)| async {
            let data = Arc::new(data);
            future::try_join_all(identifiers.iter().map(|identifier| async {
                let wrapped_key = found_keys.get(identifier).cloned().unwrap();
                wallet_user_hsm
                    .sign_wrapped(wrapped_key, Arc::clone(&data))
                    .await
                    .map(DerSignature::from)
            }))
            .await
        }))
        .await?;

        Ok(SignResult { signatures })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use p256::ecdsa::{signature::Verifier, SigningKey};
    use rand::rngs::OsRng;

    use wallet_common::{
        account::messages::instructions::{CheckPin, GenerateKey, Sign},
        utils::random_bytes,
    };
    use wallet_provider_domain::{
        model::{hsm::mock::MockPkcs11Client, wallet_user, wrapped_key::WrappedKey},
        repository::MockTransaction,
        FixedUuidGenerator,
    };
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use crate::instructions::HandleInstruction;

    #[tokio::test]
    async fn should_handle_checkpin() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let instruction = CheckPin {};
        instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &MockTransactionalWalletUserRepository::new(),
                &MockPkcs11Client::default(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_generate_key() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let instruction = GenerateKey {
            identifiers: vec!["key1".to_string(), "key2".to_string()],
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo.expect_save_keys().returning(|_, _| Ok(()));

        let result = instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &wallet_user_repo,
                &MockPkcs11Client::default(),
            )
            .await
            .unwrap();

        let generated_keys: Vec<String> = result
            .public_keys
            .into_iter()
            .map(|(identifier, _key)| identifier)
            .collect();
        assert_eq!(vec!["key1", "key2"], generated_keys);
    }

    #[tokio::test]
    async fn should_handle_sign() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let random_msg_1 = random_bytes(32);
        let random_msg_2 = random_bytes(32);
        let instruction = Sign {
            messages_with_identifiers: vec![
                (random_msg_1.clone(), vec!["key1".to_string(), "key2".to_string()]),
                (random_msg_2.clone(), vec!["key2".to_string()]),
            ],
        };
        let signing_key_1 = SigningKey::random(&mut OsRng);
        let signing_key_2 = SigningKey::random(&mut OsRng);
        let signing_key_1_bytes = signing_key_1.to_bytes().to_vec();
        let signing_key_2_bytes = signing_key_2.to_bytes().to_vec();

        let pkcs11_client = MockPkcs11Client::default();

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();

        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));

        wallet_user_repo
            .expect_find_keys_by_identifiers()
            .withf(|_, _, key_identifiers| key_identifiers.contains(&"key1".to_string()))
            .return_once(move |_, _, _| {
                Ok(HashMap::from([
                    ("key1".to_string(), WrappedKey::new(signing_key_1_bytes)),
                    ("key2".to_string(), WrappedKey::new(signing_key_2_bytes)),
                ]))
            });

        let result = instruction
            .handle(&wallet_user, &FixedUuidGenerator, &wallet_user_repo, &pkcs11_client)
            .await
            .unwrap();

        signing_key_1
            .verifying_key()
            .verify(&random_msg_1, &result.signatures[0][0].0)
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_1, &result.signatures[0][1].0)
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_2, &result.signatures[1][0].0)
            .unwrap();
    }
}
