use crypto::keys::EcdsaKey;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_account::messages::instructions::ChangePinCommit;
use wallet_account::messages::instructions::ChangePinRollback;
use wallet_account::messages::instructions::ChangePinStart;
use wallet_account::messages::registration::WalletCertificate;

use crate::account_provider::AccountProviderClient;
use crate::errors::AccountProviderError;
use crate::errors::AccountProviderResponseError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionClientFactory;
use crate::instruction::InstructionError;
use crate::pin::change::ChangePinClient;
use crate::pin::change::ChangePinClientError;
use crate::pin::key::PinKey;
use crate::storage::Storage;

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

impl<S, AK, GK, A> ChangePinClient for InstructionClientFactory<S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    type Error = InstructionError;

    async fn start_new_pin(
        &self,
        old_pin: &str,
        new_pin: &str,
        new_pin_salt: &[u8],
    ) -> Result<WalletCertificate, Self::Error> {
        let new_pin_key = PinKey {
            pin: new_pin,
            salt: new_pin_salt,
        };

        let client: InstructionClient<S, AK, GK, A> = self.create(old_pin.to_string());

        client
            .construct_and_send(|challenge| async move {
                let new_pin_key_pop = new_pin_key
                    .try_sign(&challenge)
                    .await
                    .map_err(|e| InstructionError::Signing(wallet_account::error::EncodeError::Signing(e.into())))?;
                let instruction = ChangePinStart {
                    pin_pubkey: new_pin_key
                        .verifying_key()
                        .map_err(|e| InstructionError::Signing(wallet_account::error::EncodeError::Signing(e.into())))?
                        .into(),
                    pop_pin_pubkey: new_pin_key_pop.into(),
                };
                Ok(instruction)
            })
            .await
    }

    async fn commit_new_pin(&self, new_pin: &str) -> Result<(), Self::Error> {
        let client: InstructionClient<S, AK, GK, A> = self.create(new_pin.to_string());
        client.send(ChangePinCommit {}).await
    }

    async fn rollback_new_pin(&self, old_pin: &str) -> Result<(), Self::Error> {
        let client: InstructionClient<S, AK, GK, A> = self.create(old_pin.to_string());
        client.send(ChangePinRollback {}).await
    }
}
