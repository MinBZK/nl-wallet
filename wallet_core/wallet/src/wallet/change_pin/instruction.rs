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
            InstructionError::ServerError(AccountProviderError::Response(AccountProviderResponseError::Account(
                _,
                _,
            ))) => false,
            InstructionError::ServerError(_) => true,
            InstructionError::Timeout { .. } => true,
            _ => false,
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

        let begin_result = client
            .construct_and_send(|challenge| async move {
                let new_pin_key_pop = new_pin_key
                    .try_sign(&challenge)
                    .await
                    .map_err(|e| InstructionError::Signing(AccountError::Signing(e.into())))?;
                let instruction = ChangePinStart {
                    pin_pubkey: new_pin_key
                        .verifying_key()
                        .map_err(|e| InstructionError::Signing(AccountError::Signing(e.into())))?
                        .into(), // TODO error handling
                    pop_pin_pubkey: new_pin_key_pop.into(),
                };
                Ok(instruction)
            })
            .await;

        begin_result.map(|c| c.certificate)
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
