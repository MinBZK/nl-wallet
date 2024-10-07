use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::{
        errors::Error as AccountError,
        messages::{
            auth::WalletCertificate,
            instructions::{ChangePinCommit, ChangePinRollback, ChangePinStart},
        },
    },
    keys::EcdsaKey,
};

use crate::{
    account_provider::AccountProviderClient,
    errors::{AccountProviderError, AccountProviderResponseError},
    instruction::{InstructionClient, InstructionClientFactory, InstructionError},
    pin::{
        change::{ChangePinClient, ChangePinClientError},
        key::PinKey,
    },
    storage::Storage,
};

impl ChangePinClientError for InstructionError {
    fn is_network_error(&self) -> bool {
        match self {
            Self::ServerError(error) => error.is_network_error(),
            Self::Timeout { .. } => true,
            Self::IncorrectPin { .. } => false,
            Self::Blocked => false,
            Self::InstructionValidation => false,
            Self::Signing(_) => false,
            Self::InstructionResultValidation(_) => false,
            Self::StoreInstructionSequenceNumber(_) => false,
        }
    }
}

impl ChangePinClientError for AccountProviderError {
    fn is_network_error(&self) -> bool {
        match self {
            Self::Response(error) => error.is_network_error(),
            Self::Networking(_) => true,
            Self::BaseUrl(_) => false,
        }
    }
}

/// Classifies any status codes as network error.
impl ChangePinClientError for AccountProviderResponseError {
    fn is_network_error(&self) -> bool {
        match self {
            Self::Status(..) => true,
            Self::Text(..) => true,
            Self::Account(..) => false,
        }
    }
}

impl<'a, S, K, A> ChangePinClient for InstructionClientFactory<'a, S, K, A>
where
    S: Storage,
    K: PlatformEcdsaKey,
    A: AccountProviderClient,
{
    type Error = InstructionError;

    async fn start_new_pin(
        &self,
        old_pin: &str,
        new_pin: &str,
        new_pin_salt: &[u8],
    ) -> Result<WalletCertificate, Self::Error> {
        let new_pin_key = PinKey::new(new_pin, new_pin_salt);

        let client: InstructionClient<S, K, A> = self.create(old_pin.to_string());

        client
            .construct_and_send(|challenge| async move {
                let new_pin_key_pop = new_pin_key
                    .try_sign(&challenge)
                    .await
                    .map_err(|e| InstructionError::Signing(AccountError::Signing(e.into())))?;
                let instruction = ChangePinStart {
                    pin_pubkey: new_pin_key
                        .verifying_key()
                        .map_err(|e| InstructionError::Signing(AccountError::Signing(e.into())))?
                        .into(),
                    pop_pin_pubkey: new_pin_key_pop.into(),
                };
                Ok(instruction)
            })
            .await
    }

    async fn commit_new_pin(&self, new_pin: &str) -> Result<(), Self::Error> {
        let client: InstructionClient<S, K, A> = self.create(new_pin.to_string());
        client.send(ChangePinCommit {}).await
    }

    async fn rollback_new_pin(&self, old_pin: &str) -> Result<(), Self::Error> {
        let client: InstructionClient<S, K, A> = self.create(old_pin.to_string());
        client.send(ChangePinRollback {}).await
    }
}
