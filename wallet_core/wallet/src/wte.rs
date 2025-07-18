use p256::ecdsa::VerifyingKey;

use jwt::credential::JwtCredential;
use jwt::wte::WteClaims;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_account::messages::instructions::IssueWte;
use wallet_account::messages::instructions::IssueWteResult;

use crate::account_provider::AccountProviderClient;
use crate::instruction::InstructionClient;
use crate::instruction::RemoteEcdsaKey;
use crate::storage::Storage;
use crate::wallet::IssuanceError;

pub trait WteIssuanceClient {
    async fn obtain_wte<S, AK, GK, A>(
        &self,
        wte_issuer_pubkey: &VerifyingKey,
        remote_instruction: InstructionClient<S, AK, GK, A>,
    ) -> Result<JwtCredential<WteClaims>, IssuanceError>
    where
        S: Storage,
        AK: AppleAttestedKey,
        GK: GoogleAttestedKey,
        A: AccountProviderClient;
}

pub struct WpWteIssuanceClient;

impl WteIssuanceClient for WpWteIssuanceClient {
    async fn obtain_wte<S, AK, GK, A>(
        &self,
        wte_issuer_pubkey: &VerifyingKey,
        remote_instruction: InstructionClient<S, AK, GK, A>,
    ) -> Result<JwtCredential<WteClaims>, IssuanceError>
    where
        S: Storage,
        AK: AppleAttestedKey,
        GK: GoogleAttestedKey,
        A: AccountProviderClient,
    {
        let IssueWteResult { key_id, wte } = remote_instruction.send(IssueWte).await?;
        let (wte, _) = JwtCredential::new::<RemoteEcdsaKey<S, AK, GK, A>>(key_id, wte, wte_issuer_pubkey)?;
        Ok(wte)
    }
}

impl Default for WpWteIssuanceClient {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use p256::ecdsa::VerifyingKey;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::utils::random_string;
    use jwt::credential::JwtCredential;
    use jwt::wte::WteClaims;
    use platform_support::attested_key::AppleAttestedKey;
    use platform_support::attested_key::GoogleAttestedKey;

    use crate::account_provider::AccountProviderClient;
    use crate::instruction::InstructionClient;
    use crate::storage::Storage;
    use crate::wallet::IssuanceError;

    use super::WteIssuanceClient;

    pub struct MockWteIssuanceClient;

    impl WteIssuanceClient for MockWteIssuanceClient {
        async fn obtain_wte<S, AK, GK, A>(
            &self,
            _pubkey: &VerifyingKey,
            _remote_instruction: InstructionClient<S, AK, GK, A>,
        ) -> Result<JwtCredential<WteClaims>, IssuanceError>
        where
            S: Storage,
            AK: AppleAttestedKey,
            GK: GoogleAttestedKey,
            A: AccountProviderClient,
        {
            let key_id = random_string(32);
            MockRemoteEcdsaKey::new_random(key_id.clone());
            let cred = JwtCredential::new_unverified::<MockRemoteEcdsaKey>(key_id, "header.body.signature".into());

            Ok(cred)
        }
    }

    impl Default for MockWteIssuanceClient {
        fn default() -> Self {
            Self
        }
    }
}
