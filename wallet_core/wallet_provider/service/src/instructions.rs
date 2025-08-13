use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroUsize;
use std::sync::Arc;

use base64::prelude::*;
use futures::future;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use crypto::keys::EcdsaKey;
use crypto::p256_der::DerSignature;
use hsm::model::encrypter::Encrypter;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use jwt::Jwt;
use jwt::jwk::jwk_from_p256;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteDisclosure;
use openid4vc::credential::OPENID4VCI_VC_POP_JWT_TYPE;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::ChangePinCommit;
use wallet_account::messages::instructions::ChangePinRollback;
use wallet_account::messages::instructions::ChangePinStart;
use wallet_account::messages::instructions::CheckPin;
use wallet_account::messages::instructions::PerformIssuance;
use wallet_account::messages::instructions::PerformIssuanceResult;
use wallet_account::messages::instructions::PerformIssuanceWithWua;
use wallet_account::messages::instructions::PerformIssuanceWithWuaResult;
use wallet_account::messages::instructions::Sign;
use wallet_account::messages::instructions::SignResult;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;
use wscd::POA_JWT_TYP;
use wscd::Poa;

use crate::account_server::InstructionError;
use crate::account_server::InstructionValidationError;
use crate::account_server::UserState;
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
impl ValidateInstruction for PerformIssuance {}
impl ValidateInstruction for PerformIssuanceWithWua {}

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
        user_state: &UserState<R, H, impl WteIssuer>,
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
        _user_state: &UserState<R, H, impl WteIssuer>,
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
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let tx = user_state.repositories.begin_transaction().await?;

        user_state
            .repositories
            .commit_pin_change(&tx, wallet_user.wallet_id.as_str())
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

struct IssuanceArguments {
    key_count: NonZeroUsize,
    aud: String,
    nonce: Option<String>,
    issue_wua: bool,
}

/// Helper for the [`PerformIssuance`] and [`PerformIssuanceWithWua`] instruction handlers.
async fn perform_issuance<T, R, H>(
    arguments: IssuanceArguments,
    wallet_user: &WalletUser,
    uuid_generator: &impl Generator<Uuid>,
    user_state: &UserState<R, H, impl WteIssuer>,
) -> Result<
    (
        VecNonEmpty<String>,
        VecNonEmpty<Jwt<JwtPopClaims>>,
        Option<Poa>,
        Option<WteDisclosure>,
    ),
    InstructionError,
>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let (key_ids, wrapped_keys): (Vec<_>, Vec<_>) = user_state
        .wallet_user_hsm
        .generate_wrapped_keys(&user_state.wrapping_key_identifier, arguments.key_count)
        .await?
        .into_inner()
        .into_iter()
        .multiunzip();

    // Instantiate some VecNonEmpty's that we need below. Safe because generate_wrapped_keys() returns VecNonEmpty.
    let key_ids: VecNonEmpty<_> = key_ids.try_into().unwrap();
    let attestation_keys = wrapped_keys
        .iter()
        .map(|wrapped_key| attestation_key(wrapped_key, user_state))
        .collect_vec()
        .try_into()
        .unwrap();

    // The JWT claims to be signed in the PoPs and the PoA.
    let claims = JwtPopClaims::new(arguments.nonce, NL_WALLET_CLIENT_ID.to_string(), arguments.aud);

    let (wua_key_and_id, wua_disclosure) = if arguments.issue_wua {
        let (key, key_id, wua_disclosure) = wua(&claims, user_state).await?;
        (Some((key, key_id)), Some(wua_disclosure))
    } else {
        (None, None)
    };

    let pops = issuance_pops(&attestation_keys, &claims).await?;

    let key_count_including_wua = if arguments.issue_wua {
        arguments.key_count.get() + 1
    } else {
        arguments.key_count.get()
    };
    let poa = if key_count_including_wua > 1 {
        let wua_attestation_key = wua_key_and_id.as_ref().map(|(key, _)| attestation_key(key, user_state));
        Some(
            // Unwrap is safe because we're operating on the output of `generate_wrapped_keys()`
            // which returns `VecNonEmpty`
            Poa::new(
                attestation_keys
                    .iter()
                    .chain(wua_attestation_key.as_ref())
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                claims,
            )
            .await?,
        )
    } else {
        None
    };

    // Assemble the keys to be stored in the database
    let db_keys = wrapped_keys
        .into_iter()
        .zip(key_ids.clone())
        .chain(wua_key_and_id.into_iter())
        .map(|(key, key_identifier)| WalletUserKey {
            wallet_user_key_id: uuid_generator.generate(),
            key_identifier,
            key,
        })
        .collect();

    // Save the keys in the database
    let tx = user_state.repositories.begin_transaction().await?;
    user_state
        .repositories
        .save_keys(
            &tx,
            WalletUserKeys {
                wallet_user_id: wallet_user.id,
                keys: db_keys,
            },
        )
        .await?;
    tx.commit().await?;

    Ok((key_ids, pops, poa, wua_disclosure))
}

async fn wua<T, R, H>(
    claims: &JwtPopClaims,
    user_state: &UserState<R, H, impl WteIssuer>,
) -> Result<(WrappedKey, String, WteDisclosure), InstructionError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let (wua_wrapped_key, wua_key_id, wua) = user_state
        .wte_issuer
        .issue_wte()
        .await
        .map_err(|e| InstructionError::WteIssuance(Box::new(e)))?;

    let wua_disclosure = Jwt::sign(
        claims,
        &Header::new(Algorithm::ES256),
        &attestation_key(&wua_wrapped_key, user_state),
    )
    .await
    .map_err(InstructionError::PopSigning)?;

    Ok((wua_wrapped_key, wua_key_id, WteDisclosure::new(wua, wua_disclosure)))
}

async fn issuance_pops<H>(
    attestation_keys: &VecNonEmpty<HsmCredentialSigningKey<'_, H>>,
    claims: &JwtPopClaims,
) -> Result<VecNonEmpty<Jwt<JwtPopClaims>>, InstructionError>
where
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    let pops = future::try_join_all(attestation_keys.iter().map(|attestation_key| async {
        let public_key = attestation_key.verifying_key().await?;
        let header = Header {
            typ: Some(OPENID4VCI_VC_POP_JWT_TYPE.to_string()),
            alg: Algorithm::ES256,
            jwk: Some(jwk_from_p256(&public_key)?),
            ..Default::default()
        };

        let jwt = Jwt::sign(claims, &header, attestation_key)
            .await
            .map_err(InstructionError::PopSigning)?;

        Ok::<_, InstructionError>(jwt)
    }))
    .await?
    .try_into()
    .unwrap(); // Safe because we're iterating over attestation_keys which is VecNonEmpty

    Ok(pops)
}

fn attestation_key<'a, T, R, H>(
    wrapped_key: &'a WrappedKey,
    user_state: &'a UserState<R, H, impl WteIssuer>,
) -> HsmCredentialSigningKey<'a, H>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
    H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
{
    HsmCredentialSigningKey {
        hsm: &user_state.wallet_user_hsm,
        wrapped_key,
        wrapping_key_identifier: &user_state.wrapping_key_identifier,
    }
}

impl HandleInstruction for PerformIssuance {
    type Result = PerformIssuanceResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (key_identifiers, pops, poa, _) = perform_issuance(
            IssuanceArguments {
                key_count: self.key_count,
                aud: self.aud,
                nonce: self.nonce,
                issue_wua: false,
            },
            wallet_user,
            uuid_generator,
            user_state,
        )
        .await?;

        Ok(PerformIssuanceResult {
            key_identifiers,
            pops,
            poa,
        })
    }
}

impl HandleInstruction for PerformIssuanceWithWua {
    type Result = PerformIssuanceWithWuaResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        uuid_generator: &impl Generator<Uuid>,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<Self::Result, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (key_identifiers, pops, poa, wua_with_disclosure) = perform_issuance(
            IssuanceArguments {
                key_count: self.issuance_instruction.key_count,
                aud: self.issuance_instruction.aud,
                nonce: self.issuance_instruction.nonce,
                issue_wua: true,
            },
            wallet_user,
            uuid_generator,
            user_state,
        )
        .await?;

        Ok(PerformIssuanceWithWuaResult {
            issuance_result: PerformIssuanceResult {
                key_identifiers,
                pops,
                poa,
            },
            // unwrap: `perform_issuance()` included a WUA since we passed it `true` above.
            wua_disclosure: wua_with_disclosure.unwrap(),
        })
    }
}

impl HandleInstruction for Sign {
    type Result = SignResult;

    async fn handle<T, R, H>(
        self,
        wallet_user: &WalletUser,
        _uuid_generator: &impl Generator<Uuid>,
        user_state: &UserState<R, H, impl WteIssuer>,
    ) -> Result<SignResult, InstructionError>
    where
        T: Committable,
        R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
        H: Encrypter<VerifyingKey, Error = HsmError> + WalletUserHsm<Error = HsmError>,
    {
        let (data, identifiers): (Vec<_>, Vec<_>) = self.messages_with_identifiers.into_iter().unzip();

        let tx = user_state.repositories.begin_transaction().await?;
        let found_keys = user_state
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
                user_state
                    .wallet_user_hsm
                    .sign_wrapped(&user_state.wrapping_key_identifier, wrapped_key, Arc::clone(&data))
                    .await
                    .map(DerSignature::from)
            }))
            .await
        }))
        .await?;

        // A PoA should be generated only if the unique keys, i.e. the keys referenced in the instruction
        // after deduplication, count two or more.
        if found_keys.len() < 2 {
            Ok(SignResult { signatures, poa: None })
        } else {
            // We have to feed a Vec of references to `Poa::new()`, so we need to iterate twice to construct that.
            let keys = found_keys
                .values()
                .map(|wrapped_key| attestation_key(wrapped_key, user_state))
                .collect_vec();
            let keys = keys.iter().collect_vec().try_into().unwrap(); // We know there are at least two keys
            let claims = JwtPopClaims::new(self.poa_nonce, NL_WALLET_CLIENT_ID.to_string(), self.poa_aud);
            let poa = Poa::new(keys, claims).await?;

            Ok(SignResult {
                signatures,
                poa: Some(poa),
            })
        }
    }
}

struct HsmCredentialSigningKey<'a, H> {
    hsm: &'a H,
    wrapped_key: &'a WrappedKey,
    wrapping_key_identifier: &'a str,
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
            .sign_wrapped(
                self.wrapping_key_identifier,
                self.wrapped_key.clone(),
                Arc::new(msg.to_vec()),
            )
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
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use itertools::Itertools;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::Validation;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Verifier;
    use rand::rngs::OsRng;
    use rstest::rstest;

    use crypto::utils::random_bytes;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::Jwt;
    use jwt::jwk::jwk_to_p256;
    use jwt::pop::JwtPopClaims;
    use jwt::validations;
    use jwt::wte::WteDisclosure;
    use wallet_account::NL_WALLET_CLIENT_ID;
    use wallet_account::messages::instructions::CheckPin;
    use wallet_account::messages::instructions::PerformIssuance;
    use wallet_account::messages::instructions::PerformIssuanceWithWua;
    use wallet_account::messages::instructions::Sign;
    use wallet_provider_domain::FixedUuidGenerator;
    use wallet_provider_domain::model::wallet_user;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;
    use wscd::Poa;
    use wscd::PoaPayload;

    use crate::account_server::InstructionValidationError;
    use crate::account_server::mock;
    use crate::instructions::HandleInstruction;
    use crate::instructions::ValidateInstruction;
    use crate::instructions::is_poa_message;
    use crate::wallet_certificate::mock::setup_hsm;

    #[tokio::test]
    async fn should_handle_checkpin() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my_wrapping_key_identifier";

        let instruction = CheckPin {};
        instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &mock::user_state(
                    MockTransactionalWalletUserRepository::new(),
                    setup_hsm().await,
                    wrapping_key_identifier.to_string(),
                ),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_handle_sign() {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let random_msg_1 = random_bytes(32);
        let random_msg_2 = random_bytes(32);
        let instruction = Sign {
            messages_with_identifiers: vec![
                (random_msg_1.clone(), vec!["key1".to_string(), "key2".to_string()]),
                (random_msg_2.clone(), vec!["key2".to_string()]),
            ],
            poa_nonce: Some("nonce".to_string()),
            poa_aud: "aud".to_string(),
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
                &mock::user_state(wallet_user_repo, setup_hsm().await, wrapping_key_identifier.to_string()),
            )
            .await
            .unwrap();

        signing_key_1
            .verifying_key()
            .verify(&random_msg_1, result.signatures[0][0].as_inner())
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_1, result.signatures[0][1].as_inner())
            .unwrap();
        signing_key_2
            .verifying_key()
            .verify(&random_msg_2, result.signatures[1][0].as_inner())
            .unwrap();

        let mut validations = validations();
        validations.set_audience(&["aud"]);
        validations.set_issuer(&[NL_WALLET_CLIENT_ID.to_string()]);

        Vec::<Jwt<PoaPayload>>::from(result.poa.unwrap())
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
            poa_nonce: Some("nonce".to_string()),
            poa_aud: "aud".to_string(),
        };

        let err = instruction.validate_instruction(&wallet_user).unwrap_err();
        assert_matches!(err, InstructionValidationError::PoaMessage);
    }

    async fn perform_issuance<R, I: HandleInstruction<Result = R>>(instruction: I) -> R {
        let wallet_user = wallet_user::mock::wallet_user_1();
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo.expect_save_keys().returning(|_, _| Ok(()));

        instruction
            .handle(
                &wallet_user,
                &FixedUuidGenerator,
                &mock::user_state(wallet_user_repo, setup_hsm().await, wrapping_key_identifier.to_string()),
            )
            .await
            .unwrap()
    }

    fn validate_issuance(pops: &[Jwt<JwtPopClaims>], poa: Option<Poa>, wua_with_disclosure: Option<&WteDisclosure>) {
        let mut validations = Validation::new(Algorithm::ES256);
        validations.required_spec_claims = HashSet::default();
        validations.set_issuer(&[NL_WALLET_CLIENT_ID]);
        validations.set_audience(&[POP_AUD]);

        let keys = pops
            .iter()
            .map(|pop| {
                let pubkey = jwk_to_p256(&jsonwebtoken::decode_header(&pop.0).unwrap().jwk.unwrap()).unwrap();

                pop.parse_and_verify(&(&pubkey).into(), &validations).unwrap();

                pubkey
            })
            .collect_vec();

        let wua_key = wua_with_disclosure.map(|wua_with_disclosure| {
            let wua_key = jwk_to_p256(
                &wua_with_disclosure
                    .wte()
                    .dangerous_parse_unverified()
                    .unwrap()
                    .1
                    .confirmation
                    .jwk,
            )
            .unwrap();

            wua_with_disclosure
                .wte_pop()
                .parse_and_verify(&((&wua_key).into()), &validations)
                .unwrap();

            wua_key
        });

        let keys = keys.into_iter().chain(wua_key).collect_vec();
        if keys.len() > 1 {
            poa.unwrap()
                .verify(&keys, POP_AUD, &[NL_WALLET_CLIENT_ID.to_string()], POP_NONCE)
                .unwrap();
        }
    }

    const POP_AUD: &str = "aud";
    const POP_NONCE: &str = "nonce";

    #[tokio::test]
    #[rstest]
    #[case(1)]
    #[case(2)]
    async fn should_handle_perform_issuance(#[case] key_count: usize) {
        let result = perform_issuance(PerformIssuance {
            key_count: key_count.try_into().unwrap(),
            aud: POP_AUD.to_string(),
            nonce: Some(POP_NONCE.to_string()),
        })
        .await;

        validate_issuance(result.pops.as_slice(), result.poa, None);
    }

    #[tokio::test]
    async fn should_handle_perform_issuance_with_wua() {
        let result = perform_issuance(PerformIssuanceWithWua {
            issuance_instruction: PerformIssuance {
                key_count: 1.try_into().unwrap(),
                aud: POP_AUD.to_string(),
                nonce: Some(POP_NONCE.to_string()),
            },
        })
        .await;

        validate_issuance(
            result.issuance_result.pops.as_slice(),
            result.issuance_result.poa,
            Some(&result.wua_disclosure),
        );
    }
}
