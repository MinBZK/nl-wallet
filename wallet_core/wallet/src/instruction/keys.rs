use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroUsize;

use crypto::WithVerifyingKey;
use derive_more::Constructor;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature;

use crypto::keys::CredentialEcdsaKey;
use crypto::keys::WithIdentifier;
use crypto::p256_der::DerSignature;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_account::messages::instructions::PerformIssuance;
use wallet_account::messages::instructions::PerformIssuanceWithWua;
use wallet_account::messages::instructions::PerformIssuanceWithWuaResult;
use wallet_account::messages::instructions::Sign;
use wscd::Poa;
use wscd::wscd::IssuanceResult;
use wscd::wscd::Wscd;

use crate::account_provider::AccountProviderClient;
use crate::storage::Storage;

use super::InstructionClient;
use super::InstructionError;

#[derive(Debug, thiserror::Error)]
pub enum RemoteEcdsaKeyError {
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    Signature(#[from] signature::Error),
    #[error("no signature received from Wallet Provider")]
    MissingSignature,
    #[error("key '{0}' not found in Wallet Provider")]
    KeyNotFound(String),
}

#[derive(Constructor)]
pub struct RemoteEcdsaWscd<S, AK, GK, A> {
    instruction_client: InstructionClient<S, AK, GK, A>,
}

pub struct RemoteEcdsaKey {
    identifier: String,
    public_key: VerifyingKey,
}

impl PartialEq for RemoteEcdsaKey {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for RemoteEcdsaKey {}

impl Hash for RemoteEcdsaKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl<S, AK, GK, A> DisclosureWscd for RemoteEcdsaWscd<S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    type Key = RemoteEcdsaKey;
    type Error = RemoteEcdsaKeyError;
    type Poa = Poa;

    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        RemoteEcdsaKey {
            identifier: identifier.into(),
            public_key,
        }
    }

    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        poa_input: <Self::Poa as WscdPoa>::Input,
    ) -> Result<DisclosureResult<Self::Poa>, Self::Error> {
        let sign_result = self
            .instruction_client
            .send(Sign {
                messages_with_identifiers: messages_and_keys
                    .into_iter()
                    .map(|(message, keys)| {
                        let identifiers = keys.into_iter().map(|key| key.identifier.clone()).collect();
                        (message, identifiers)
                    })
                    .collect(),
                poa_aud: poa_input.aud,
                poa_nonce: poa_input.nonce,
            })
            .await?;

        let signatures = sign_result
            .signatures
            .into_iter()
            .map(|signatures| signatures.into_iter().map(DerSignature::into_inner).collect())
            .collect();

        Ok(DisclosureResult::new(signatures, sign_result.poa))
    }
}

impl<S, AK, GK, A> Wscd for RemoteEcdsaWscd<S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    async fn perform_issuance(
        &self,
        key_count: NonZeroUsize,
        aud: String,
        nonce: Option<String>,
        include_wua: bool,
    ) -> Result<IssuanceResult<Self::Poa>, Self::Error> {
        let issuance_instruction = PerformIssuance { key_count, aud, nonce };
        let (issuance_result, wua) = if !include_wua {
            (self.instruction_client.send(issuance_instruction).await?, None)
        } else {
            let PerformIssuanceWithWuaResult {
                issuance_result,
                wua_disclosure,
            } = self
                .instruction_client
                .send(PerformIssuanceWithWua { issuance_instruction })
                .await?;

            (issuance_result, Some(wua_disclosure))
        };

        Ok(IssuanceResult::new(
            issuance_result.key_identifiers,
            issuance_result.pops,
            issuance_result.poa,
            wua,
        ))
    }
}

impl WithIdentifier for RemoteEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl WithVerifyingKey for RemoteEcdsaKey {
    type Error = RemoteEcdsaKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(self.public_key)
    }
}

impl CredentialEcdsaKey for RemoteEcdsaKey {}
