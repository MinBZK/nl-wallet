use async_trait::async_trait;
use p256::ecdsa::{Signature, SigningKey};
use rand::rngs::OsRng;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use wallet_common::{
    account::{
        messages::instructions::{CheckPin, GenerateKey, GenerateKeyResult, Sign, SignResult},
        serialization::{DerSignature, DerVerifyingKey},
    },
    generator::Generator,
};
use wallet_provider_domain::{
    model::wallet_user::WalletUser,
    repository::{Committable, TransactionStarter, WalletUserRepository},
};

use crate::account_server::InstructionError;

#[async_trait]
pub trait HandleInstruction {
    type Result: Serialize;

    async fn handle<T>(
        &self,
        wallet_user: &WalletUser,
        uuid_generator: &(impl Generator<Uuid> + Sync),
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable + Send + Sync;
}

#[async_trait]
impl HandleInstruction for CheckPin {
    type Result = ();

    async fn handle<T>(
        &self,
        _wallet_user: &WalletUser,
        _uuid_generator: &(impl Generator<Uuid> + Sync),
        _repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
    ) -> Result<(), InstructionError>
    where
        T: Committable + Send + Sync,
    {
        Ok(())
    }
}

#[async_trait]
impl HandleInstruction for GenerateKey {
    type Result = GenerateKeyResult;

    async fn handle<T>(
        &self,
        wallet_user: &WalletUser,
        uuid_generator: &(impl Generator<Uuid> + Sync),
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
    ) -> Result<GenerateKeyResult, InstructionError>
    where
        T: Committable + Send + Sync,
    {
        let tx = repositories.begin_transaction().await?;

        let keys: Vec<(Uuid, String, SigningKey)> = self
            .identifiers
            .iter()
            .map(|identifier| {
                (
                    uuid_generator.generate(),
                    identifier.clone(),
                    SigningKey::random(&mut OsRng),
                )
            })
            .collect();

        repositories.save_keys(&tx, wallet_user.id, &keys).await?;
        tx.commit().await?;

        let public_keys = keys
            .into_iter()
            .map(|(_id, identifier, key)| (identifier, DerVerifyingKey::from(*key.verifying_key())))
            .collect();

        Ok(GenerateKeyResult { public_keys })
    }
}

#[async_trait]
impl HandleInstruction for Sign {
    type Result = SignResult;

    async fn handle<T>(
        &self,
        wallet_user: &WalletUser,
        _uuid_generator: &(impl Generator<Uuid> + Sync),
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable + Send + Sync,
    {
        let (msg, identifiers) = &self.msg_with_identifiers;

        let tx = repositories.begin_transaction().await?;
        let found_keys = repositories
            .find_keys_by_identifiers(&tx, wallet_user.id, identifiers)
            .await?;
        tx.commit().await?;

        let signatures: HashMap<String, DerSignature> = found_keys
            .into_iter()
            .map(|(identifier, key)| {
                let signature: Signature = p256::ecdsa::signature::Signer::sign(&key, &msg.0);
                (identifier, signature.into())
            })
            .collect();

        Ok(SignResult {
            signatures_by_identifier: signatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::{signature::Verifier, SigningKey};
    use rand::rngs::OsRng;
    use std::collections::HashMap;

    use wallet_common::{
        account::messages::instructions::{CheckPin, GenerateKey, Sign},
        utils::random_bytes,
    };
    use wallet_provider_domain::{model::wallet_user, repository::MockTransaction, FixedUuidGenerator};
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
        wallet_user_repo.expect_save_keys().returning(|_, _, _| Ok(()));

        let result = instruction
            .handle(&wallet_user, &FixedUuidGenerator, &wallet_user_repo)
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

        let signing_key = SigningKey::random(&mut OsRng);
        let returned_signing_key = signing_key.clone();

        let instruction = Sign {
            msg_with_identifiers: (random_bytes(32).into(), vec!["key1".to_string()]),
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_keys_by_identifiers()
            .withf(|_, _, key_identifiers| key_identifiers.contains(&"key1".to_string()))
            .return_once(move |_, _, _| Ok(HashMap::from([("key1".to_string(), returned_signing_key)])));

        let result = instruction
            .handle(&wallet_user, &FixedUuidGenerator, &wallet_user_repo)
            .await
            .unwrap();

        result
            .signatures_by_identifier
            .iter()
            .for_each(|(_identifier, signature)| {
                signing_key
                    .verifying_key()
                    .verify(&instruction.msg_with_identifiers.0 .0, &signature.0)
                    .unwrap();
            })
    }
}
