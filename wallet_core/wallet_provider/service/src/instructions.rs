use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use base64::prelude::*;
use futures::future;
use itertools::Itertools;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use wallet_common::account::messages::instructions::ChangePinCommit;
use wallet_common::account::messages::instructions::ChangePinRollback;
use wallet_common::account::messages::instructions::ChangePinStart;
use wallet_common::account::messages::instructions::CheckPin;
use wallet_common::account::messages::instructions::ConstructPoa;
use wallet_common::account::messages::instructions::ConstructPoaResult;
use wallet_common::account::messages::instructions::GenerateKey;
use wallet_common::account::messages::instructions::GenerateKeyResult;
use wallet_common::account::messages::instructions::IssueWte;
use wallet_common::account::messages::instructions::IssueWteResult;
use wallet_common::account::messages::instructions::Sign;
use wallet_common::account::messages::instructions::SignResult;
use wallet_common::account::serialization::DerSignature;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::generator::Generator;
use wallet_common::jwt::JwtPopClaims;
use wallet_common::jwt::NL_WALLET_CLIENT_ID;
use wallet_common::keys::poa::Poa;
use wallet_common::keys::poa::POA_JWT_TYP;
use wallet_common::keys::EcdsaKey;
use wallet_provider_domain::model::encrypter::Encrypter;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::model::wrapped_key::WrappedKey;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::InstructionError;
use crate::account_server::InstructionState;
use crate::account_server::InstructionValidationError;
use crate::hsm::HsmError;
use crate::wte_issuer::WteIssuer;

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
impl ValidateInstruction for ConstructPoa {}

impl ValidateInstruction for Sign {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        if wallet_user.pin_change_in_progress() {
            return Err(InstructionValidationError::PinChangeInProgress);
        }

        if self
            .messages_with_identifiers
            .iter()
            .any(|(msg, _)| is_poa_message(msg))
        {
            let user = &wallet_user.id;
            warn!("user {user} attempted to sign a PoA via the Sign instruction instead of ConstructPoa");
            return Err(InstructionValidationError::PoaMessage);
        }

        Ok(())
    }
}

impl ValidateInstruction for IssueWte {
    fn validate_instruction(&self, wallet_user: &WalletUser) -> Result<(), InstructionValidationError> {
        if wallet_user.pin_change_in_progress() {
            return Err(InstructionValidationError::PinChangeInProgress);
        }

        // Since the user can exchange the WTE for the PID at the PID issuer, and since one of the purposes of the WTE
        // is ensuring that a user can have only a single PID in their wallet, we must ensure that we didn't already
        // issue a WTE at some point in the past.
        if wallet_user.has_wte {
            let user = &wallet_user.id;
            warn!("user {user} sent a second IssueWte instruction");
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
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
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
        _instruction_state: &InstructionState<R, H, impl WteIssuer>,
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
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let tx = instruction_state.repositories.begin_transaction().await?;

        instruction_state
            .repositories
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
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
    ) -> Result<GenerateKeyResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let identifiers: Vec<&str> = self.identifiers.iter().map(|i| i.as_str()).collect();
        let keys = instruction_state
            .wallet_user_hsm
            .generate_wrapped_keys(&identifiers)
            .await?;

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

        let tx = instruction_state.repositories.begin_transaction().await?;
        instruction_state
            .repositories
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
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (data, identifiers): (Vec<_>, Vec<_>) = self.messages_with_identifiers.into_iter().unzip();

        let tx = instruction_state.repositories.begin_transaction().await?;
        let found_keys = instruction_state
            .repositories
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
                instruction_state
                    .wallet_user_hsm
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
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (wrapped_privkey, wte) = instruction_state
            .wte_issuer
            .issue_wte()
            .await
            .map_err(|e| InstructionError::WteIssuance(Box::new(e)))?;

        let tx = instruction_state.repositories.begin_transaction().await?;
        let keys = WalletUserKeys {
            wallet_user_id: wallet_user.id,
            keys: vec![WalletUserKey {
                wallet_user_key_id: uuid_generator.generate(),
                key_identifier: self.key_identifier,
                key: wrapped_privkey,
            }],
        };
        instruction_state.repositories.save_keys(&tx, keys).await?;
        instruction_state
            .repositories
            .save_wte_issued(&tx, &wallet_user.wallet_id)
            .await?;
        tx.commit().await?;

        Ok(IssueWteResult { wte })
    }
}

impl HandleInstruction for ConstructPoa {
    type Result = ConstructPoaResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        instruction_state: &InstructionState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let tx = instruction_state.repositories.begin_transaction().await?;
        let mut keys = instruction_state
            .repositories
            .find_keys_by_identifiers(&tx, wallet_user.id, self.key_identifiers.as_slice())
            .await?;
        tx.commit().await?;

        let keys = self
            .key_identifiers
            .as_slice()
            .iter()
            .map(|key_identifier| {
                let wrapped_key = keys
                    .remove(key_identifier) // remove() is like get() but lets us take ownership, avoiding a clone
                    .ok_or(InstructionError::NonexistingKey(key_identifier.clone()))?;
                Ok(HsmCredentialSigningKey {
                    hsm: &instruction_state.wallet_user_hsm,
                    wrapped_key,
                })
            })
            .collect::<Result<Vec<_>, InstructionError>>()?;

        // Poa::new() needs a vec of references. We can unwrap because self.key_identifiers is a VecAtLeastTwo.
        let keys = keys.iter().collect_vec().try_into().unwrap();
        let claims = JwtPopClaims::new(self.nonce, NL_WALLET_CLIENT_ID.to_string(), self.aud);
        let poa = Poa::new(keys, claims).await?;

        Ok(ConstructPoaResult { poa })
    }
}

struct HsmCredentialSigningKey<'a, H> {
    hsm: &'a H,
    wrapped_key: WrappedKey,
}

impl<H> PartialEq for HsmCredentialSigningKey<'_, H> {
    fn eq(&self, other: &Self) -> bool {
        self.wrapped_key == other.wrapped_key
    }
}

impl<H> Eq for HsmCredentialSigningKey<'_, H> {}

impl<H> Hash for HsmCredentialSigningKey<'_, H> {
    fn hash<HASH: Hasher>(&self, state: &mut HASH) {
        self.wrapped_key.hash(state);
    }
}

impl<H> EcdsaKey for HsmCredentialSigningKey<'_, H>
where
    H: WalletUserHsm<Error = HsmError>,
{
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.wrapped_key.public_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.hsm
            .sign_wrapped(self.wrapped_key.clone(), Arc::new(msg.to_vec()))
            .await
    }
}

/// Test if the `message` is the payload of a PoA JWT, i.e. `header.body` where `header` is the base64url encoding
/// of a JSON object containing `"typ":"poa+jwt"`.
///
/// This function must be used by the signing instructions to prevent a user from signing a PoA without
/// using the intended `ConstructPoa` instruction for that. It should therefore be resistant to "tricks"
/// such as include whitespace in the JSON or mess with the casing of the casing of the value of the `typ` field.
///
/// Since this function is executed for every single message that the WP signs for a wallet, before JSON deserialization
/// of the header we do a number of cheaper checks to return early if the passed message is clearly not a PoA JWT
/// payload.
fn is_poa_message(message: &[u8]) -> bool {
    // A JWT payload contains a single dot which is not located at the beginning of the string.
    let predicate = |&x| x == b'.';
    let dot_pos = match message.iter().position(predicate) {
        None | Some(0) => return false, // a string without dot, or whose first character is a dot is not a JWT payload
        Some(dot_pos) => {
            if message.iter().skip(dot_pos + 1).any(predicate) {
                return false; // a string with more than one dot is not a JWT payload
            }

            dot_pos
        }
    };

    let first_part = &message[0..dot_pos];
    let Ok(decoded) = BASE64_URL_SAFE_NO_PAD.decode(first_part) else {
        return false; // not a PoA in case of Base64url decoding errors
    };

    // We use a custom `Header` struct here as opposed to `jsonwebtoken::Header` so as to only deserialize
    // the `typ` field and not any of the other ones in `jsonwebtoken::Header`.
    #[derive(Deserialize)]
    struct Header {
        typ: String,
    }

    let Ok(header) = serde_json::from_slice::<Header>(&decoded) else {
        return false; // not a PoA in case of JSON deserialization errors
    };

    header.typ.to_ascii_lowercase() == POA_JWT_TYP
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use p256::ecdsa::signature::Verifier;
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;
    use rstest::rstest;

    use wallet_common::account::messages::instructions::CheckPin;
    use wallet_common::account::messages::instructions::ConstructPoa;
    use wallet_common::account::messages::instructions::GenerateKey;
    use wallet_common::account::messages::instructions::IssueWte;
    use wallet_common::account::messages::instructions::Sign;
    use wallet_common::jwt::validations;
    use wallet_common::jwt::Jwt;
    use wallet_common::keys::poa::PoaPayload;
    use wallet_common::utils::random_bytes;
    use wallet_common::utils::random_string;
    use wallet_provider_domain::model::wallet_user;
    use wallet_provider_domain::model::wallet_user::WalletUser;
    use wallet_provider_domain::model::wrapped_key::WrappedKey;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_domain::FixedUuidGenerator;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use crate::account_server::mock;
    use crate::account_server::InstructionValidationError;
    use crate::instructions::is_poa_message;
    use crate::instructions::HandleInstruction;
    use crate::instructions::ValidateInstruction;
    use crate::wallet_certificate::mock::setup_hsm;

    #[tokio::test]
    async fn should_handle_checkpin() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let instruction = CheckPin {};
        instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &mock::instruction_state(MockTransactionalWalletUserRepository::new(), setup_hsm().await),
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
                &mock::instruction_state(wallet_user_repo, setup_hsm().await),
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
        let signing_key_1_public = *signing_key_1.verifying_key();
        let signing_key_2_public = *signing_key_2.verifying_key();

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();

        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));

        wallet_user_repo
            .expect_find_keys_by_identifiers()
            .withf(|_, _, key_identifiers| key_identifiers.contains(&"key1".to_string()))
            .return_once(move |_, _, _| {
                Ok(HashMap::from([
                    (
                        "key1".to_string(),
                        WrappedKey::new(signing_key_1_bytes, signing_key_1_public),
                    ),
                    (
                        "key2".to_string(),
                        WrappedKey::new(signing_key_2_bytes, signing_key_2_public),
                    ),
                ]))
            });

        let result = instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &mock::instruction_state(wallet_user_repo, setup_hsm().await),
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
                &mock::instruction_state(wallet_user_repo, setup_hsm().await),
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

    #[tokio::test]
    async fn should_handle_construct_poa() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let signing_key_1 = SigningKey::random(&mut OsRng);
        let signing_key_2 = SigningKey::random(&mut OsRng);
        let signing_key_1_bytes = signing_key_1.to_bytes().to_vec();
        let signing_key_2_bytes = signing_key_2.to_bytes().to_vec();
        let signing_key_1_public = *signing_key_1.verifying_key();
        let signing_key_2_public = *signing_key_2.verifying_key();

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_keys_by_identifiers()
            .return_once(move |_, _, _| {
                Ok(HashMap::from([
                    (
                        "key1".to_string(),
                        WrappedKey::new(signing_key_1_bytes, signing_key_1_public),
                    ),
                    (
                        "key2".to_string(),
                        WrappedKey::new(signing_key_2_bytes, signing_key_2_public),
                    ),
                ]))
            });

        let instruction = ConstructPoa {
            key_identifiers: vec!["key1".to_string(), "key2".to_string()].try_into().unwrap(),
            aud: "aud".to_string(),
            nonce: None,
        };

        let poa = instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &mock::instruction_state(wallet_user_repo, setup_hsm().await),
            )
            .await
            .unwrap()
            .poa;

        let mut validations = validations();
        validations.set_audience(&["aud"]);

        Vec::<Jwt<PoaPayload>>::from(poa)
            .into_iter()
            .zip([signing_key_1_public, signing_key_2_public])
            .for_each(|(jwt, pubkey)| {
                jwt.parse_and_verify(&(&pubkey).into(), &validations).unwrap();
            });
    }

    fn mock_jwt_payload(header: &str) -> Vec<u8> {
        (BASE64_URL_SAFE_NO_PAD.encode(header) + "." + &BASE64_URL_SAFE_NO_PAD.encode("{}")).into_bytes()
    }

    #[rstest]
    #[case(mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{"typ":"poa+JWT"}"#), true)] // accept any casing of the field value
    #[case(mock_jwt_payload(r#"{"typ":"PoA+jWt"}"#), true)]
    #[case(mock_jwt_payload(r#"{"typ": "poa+jwt"}"#), true)] // whitespace in the JSON doesn't matter
    #[case(mock_jwt_payload(r#"{ "typ":"poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#" {"typ": "poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{ "typ": "poa+jwt"}"#), true)]
    #[case(mock_jwt_payload(r#"{	"typ":"poa+jwt"}"#), true)]
    #[case(
        mock_jwt_payload(
            r#"{"typ"
:"poa+jwt"}"#
        ),
        true
    )]
    #[case(
        mock_jwt_payload(
            r#" {	"typ":
"poa+jwt"}"#
        ),
        true
    )]
    #[case(mock_jwt_payload(r#"{"Typ":"poa+jwt"}"#), false)] // a differently cased field name is a different field
    #[case(mock_jwt_payload(r#"{" typ":"poa+jwt"}"#), false)] // whitespace in the field name is a different field
    #[case(mock_jwt_payload(r#"{"typ":" poa+jwt"}"#), false)] // or in the field value
    #[case(mock_jwt_payload(r#"{"typ":"jwt"}"#), false)] // an ordinary JWT is not a PoA
    #[case(mock_jwt_payload(r#"{"typ":42}"#), false)] // Invalid JSON is not a PoA
    #[case(mock_jwt_payload(r#"{"typ"}"#), false)]
    #[case(".blah".to_string().into_bytes(), false)]
    #[case([".".to_string().into_bytes(), mock_jwt_payload(r#"{"typ":"jwt"}"#)].concat(), false)]
    #[case([mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), ".blah".to_string().into_bytes()].concat(), false)]
    #[test]
    fn test_is_poa_message(#[case] msg: Vec<u8>, #[case] is_poa: bool) {
        assert_eq!(is_poa_message(&msg), is_poa);
    }

    #[tokio::test]
    async fn test_cannot_sign_poa_via_sign_instruction() {
        let wallet_user = wallet_user::mock::wallet_user_1();

        let instruction = Sign {
            messages_with_identifiers: vec![(mock_jwt_payload(r#"{"typ":"poa+jwt"}"#), vec!["key".to_string()])],
        };

        let err = instruction.validate_instruction(&wallet_user).unwrap_err();
        assert_matches!(err, InstructionValidationError::PoaMessage);
    }
}
