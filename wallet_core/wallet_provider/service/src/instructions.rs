use std::sync::Arc;

use futures::future::{self};
use p256::ecdsa::{Signature, VerifyingKey};
use serde::Serialize;
use uuid::Uuid;

use wallet_common::{
    account::{
        messages::instructions::{
            ChangePinCommit, ChangePinRollback, ChangePinStart, CheckPin, GenerateKey, GenerateKeyResult, IssueWte,
            IssueWteResult, NewPoa, NewPoaResult, Sign, SignResult,
        },
        serialization::{DerSignature, DerVerifyingKey},
    },
    generator::Generator,
    jwt::{JwtPopClaims, NL_WALLET_CLIENT_ID},
    keys::{poa::new_poa, EcdsaKey},
};
use wallet_provider_domain::{
    model::{
        encrypter::Encrypter,
        hsm::WalletUserHsm,
        wallet_user::{WalletId, WalletUser, WalletUserKey, WalletUserKeys},
    },
    repository::{Committable, TransactionStarter, WalletUserRepository},
};

use crate::{
    account_server::{InstructionError, InstructionValidationError},
    hsm::HsmError,
    wte_issuer::WteIssuer,
};

pub trait ValidateInstruction {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        if wallet_user.pin_change_in_progress() {
            return Err(InstructionValidationError::PinChangeInProgress);
        }

        Ok(())
    }
}

impl ValidateInstruction for CheckPin {}
impl ValidateInstruction for ChangePinStart {}
impl ValidateInstruction for GenerateKey {}
impl ValidateInstruction for Sign {}
impl ValidateInstruction for NewPoa {}

impl ValidateInstruction for IssueWte {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        // Since the user can exchange the WTE for the PID at the PID issuer, and since one of the purposes of the WTE
        // is ensuring that a user can have only a single PID in their wallet, we must ensure that we didn't already
        // issue a WTE at some point in the past.
        if wallet_user.has_wte {
            return Err(InstructionValidationError::WteAlreadyIssued);
        }

        Ok(())
    }
}

impl ValidateInstruction for ChangePinCommit {
    fn validate_instruction(&self, _wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        Ok(())
    }
}

impl ValidateInstruction for ChangePinRollback {
    fn validate_instruction(&self, _wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        Ok(())
    }
}

pub trait HandleInstruction {
    type Result: Serialize;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &H,
        wte_issuer: &impl WteIssuer,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>;
}

impl HandleInstruction for CheckPin {
    type Result = ();

    async fn handle<T, R, H>(
        self,
        _wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        _wallet_user_repository: &R,
        _wallet_user_hsm: &H,
        _wte_issuer: &impl WteIssuer,
    ) -> Result<(), InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        Ok(())
    }
}

impl HandleInstruction for ChangePinCommit {
    type Result = ();

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        _wallet_user_hsm: &H,
        _wte_issuer: &impl WteIssuer,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let tx = wallet_user_repository.begin_transaction().await?;

        wallet_user_repository
            .commit_pin_change(&tx, wallet_user.wallet_id.as_str())
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl HandleInstruction for GenerateKey {
    type Result = GenerateKeyResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &H,
        _wte_issuer: &impl WteIssuer,
    ) -> Result<GenerateKeyResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
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

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        wallet_user_hsm: &H,
        _wte_issuer: &impl WteIssuer,
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
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

impl HandleInstruction for IssueWte {
    type Result = IssueWteResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        wallet_user_repository: &R,
        _wallet_user_hsm: &H,
        wte_issuer: &impl WteIssuer,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (wrapped_privkey, wte) = wte_issuer
            .issue_wte()
            .await
            .map_err(|e| InstructionError::WteIssuance(Box::new(e)))?;

        let tx = wallet_user_repository.begin_transaction().await?;
        let keys = WalletUserKeys {
            wallet_user_id: wallet_user.id,
            keys: vec![WalletUserKey {
                wallet_user_key_id: uuid_generator.generate(),
                key_identifier: self.key_identifier,
                key: wrapped_privkey,
            }],
        };
        wallet_user_repository.save_keys(&tx, keys).await?;
        wallet_user_repository
            .save_wte_issued(&tx, &wallet_user.wallet_id)
            .await?;
        tx.commit().await?;

        Ok(IssueWteResult { wte })
    }
}

impl HandleInstruction for NewPoa {
    type Result = NewPoaResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        _wallet_user_repository: &R,
        wallet_user_hsm: &H,
        _wte_issuer: &impl WteIssuer,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let keys: Vec<_> = self
            .key_identifiers
            .iter()
            .map(|key_identifier| HsmCredentialSigningKey {
                hsm: wallet_user_hsm,
                wallet_id: &wallet_user.wallet_id,
                key_identifier,
            })
            .collect();

        let claims = JwtPopClaims::new(self.nonce, NL_WALLET_CLIENT_ID.to_string(), self.aud);
        let poa = new_poa(keys.iter().collect(), claims).await.unwrap();

        Ok(NewPoaResult { poa })
    }
}

struct HsmCredentialSigningKey<'a, H> {
    hsm: &'a H,
    wallet_id: &'a WalletId,
    key_identifier: &'a str,
}

impl<'a, H> EcdsaKey for HsmCredentialSigningKey<'a, H>
where
    H: WalletUserHsm<Error = HsmError>,
{
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.hsm.verifying_key(&self.wallet_id, &self.key_identifier).await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.hsm
            .sign(&self.wallet_id, &self.key_identifier, Arc::new(msg.to_vec()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use p256::ecdsa::{signature::Verifier, SigningKey};
    use rand::rngs::OsRng;

    use wallet_common::{
        account::messages::instructions::{CheckPin, GenerateKey, IssueWte, Sign},
        utils::{random_bytes, random_string},
    };
    use wallet_provider_domain::{
        model::{
            hsm::mock::MockPkcs11Client,
            wallet_user::{self, WalletUser},
            wrapped_key::WrappedKey,
        },
        repository::MockTransaction,
        FixedUuidGenerator,
    };
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use crate::{
        account_server::InstructionValidationError,
        instructions::{HandleInstruction, ValidateInstruction},
        wte_issuer::mock::MockWteIssuer,
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
                &MockWteIssuer,
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
                &MockWteIssuer,
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
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &wallet_user_repo,
                &pkcs11_client,
                &MockWteIssuer,
            )
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

    #[tokio::test]
    async fn should_handle_issue_wte() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let pkcs11_client = MockPkcs11Client::default();

        let instruction = IssueWte {
            key_identifier: random_string(32),
        };

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();

        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_save_wte_issued()
            .times(1)
            .return_once(|_, _| Ok(()));
        wallet_user_repo.expect_save_keys().times(1).return_once(|_, _| Ok(()));

        let result = instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &wallet_user_repo,
                &pkcs11_client,
                &MockWteIssuer,
            )
            .await
            .unwrap();

        // MockWteIssuer returns "a.b.c"
        assert!(result.wte.0.chars().filter(|c| *c == '.').count() == 2);
    }

    #[tokio::test]
    async fn should_not_issue_multiple_wtes() {
        let wallet_user = WalletUser {
            has_wte: true,
            ..wallet_user::mock::wallet_user_1()
        };

        let instruction = IssueWte {
            key_identifier: random_string(32),
        };

        let result = instruction.validate_instruction(&wallet_user).unwrap_err();

        assert_matches!(result, InstructionValidationError::WteAlreadyIssued);
    }
}
