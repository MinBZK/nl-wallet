use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
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
    model::wallet_user::WalletUser,
    repository::{Committable, TransactionStarter, WalletUserRepository},
};

use crate::{account_server::InstructionError, hsm::Pkcs11Client};

#[async_trait]
pub trait HandleInstruction {
    type Result: Serialize;

    async fn handle<T>(
        &self,
        wallet_user: &WalletUser,
        uuid_generator: &(impl Generator<Uuid> + Sync),
        repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
        pkcs11_client: &(impl Pkcs11Client + Sync),
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
        _pkcs11_client: &(impl Pkcs11Client + Sync),
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
        _uuid_generator: &(impl Generator<Uuid> + Sync),
        _repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
        pkcs11_client: &(impl Pkcs11Client + Sync),
    ) -> Result<GenerateKeyResult, InstructionError>
    where
        T: Committable + Send + Sync,
    {
        let identifiers: Vec<&str> = self.identifiers.iter().map(|i| i.as_str()).collect();
        let keys = pkcs11_client
            .generate_keys(&wallet_user.wallet_id, &identifiers)
            .await?;

        let public_keys = keys
            .into_iter()
            .map(|(identifier, key)| (identifier, DerVerifyingKey::from(key)))
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
        _repositories: &(impl TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T> + Sync),
        pkcs11_client: &(impl Pkcs11Client + Sync),
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable + Send + Sync,
    {
        let (msg, identifiers) = &self.msg_with_identifiers;
        let data = Arc::new(msg.0.clone());
        let identifiers: Vec<&str> = identifiers.iter().map(|i| i.as_str()).collect();
        let identifiers_and_signatures = pkcs11_client
            .sign_multiple(&wallet_user.wallet_id, &identifiers, data)
            .await?;

        let signatures_by_identifier: HashMap<String, DerSignature> = identifiers_and_signatures
            .into_iter()
            .map(|(identifier, signature)| (identifier, signature.into()))
            .collect();

        Ok(SignResult {
            signatures_by_identifier,
        })
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::signature::Verifier;

    use wallet_common::{
        account::messages::instructions::{CheckPin, GenerateKey, Sign},
        utils::random_bytes,
    };
    use wallet_provider_domain::{model::wallet_user, repository::MockTransaction, FixedUuidGenerator};
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use crate::{
        hsm::{mock::MockPkcs11Client, Pkcs11Client},
        instructions::HandleInstruction,
    };

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
        wallet_user_repo.expect_save_keys().returning(|_, _, _| Ok(()));

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

        let instruction = Sign {
            msg_with_identifiers: (random_bytes(32).into(), vec!["key1".to_string()]),
        };

        let pkcs11_client = MockPkcs11Client::default();
        pkcs11_client
            .generate_key(&wallet_user.wallet_id, "key1")
            .await
            .unwrap();

        let wallet_user_repo = MockTransactionalWalletUserRepository::new();

        let result = instruction
            .handle(&wallet_user, &FixedUuidGenerator, &wallet_user_repo, &pkcs11_client)
            .await
            .unwrap();

        result
            .signatures_by_identifier
            .iter()
            .for_each(|(identifier, signature)| {
                let signing_key = pkcs11_client.get_key(&wallet_user.wallet_id, identifier).unwrap();
                signing_key
                    .verifying_key()
                    .verify(&instruction.msg_with_identifiers.0 .0, &signature.0)
                    .unwrap();
            })
    }
}
