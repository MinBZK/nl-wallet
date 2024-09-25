use nl_wallet_mdoc::holder::TrustAnchor;
use openid4vc::jwt::JwtCredential;
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::messages::instructions::{IssueWte, IssueWteResult};

use crate::{
    account_provider::AccountProviderClient,
    instruction::{InstructionClient, RemoteEcdsaKey},
    storage::Storage,
    wallet::PidIssuanceError,
};

pub trait WteIssuanceClient {
    async fn obtain_wte<S, PEK, APC>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        remote_instruction: &InstructionClient<'_, S, PEK, APC>,
    ) -> Result<JwtCredential, PidIssuanceError>
    where
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient;
}

pub struct WpWteIssuanceClient;

impl WteIssuanceClient for WpWteIssuanceClient {
    async fn obtain_wte<S, PEK, APC>(
        &self,
        trust_anchors: &[TrustAnchor<'_>],
        remote_instruction: &InstructionClient<'_, S, PEK, APC>,
    ) -> Result<JwtCredential, PidIssuanceError>
    where
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient,
    {
        let IssueWteResult { key_id, wte } = remote_instruction.send(IssueWte).await?;
        let (wte, _) = JwtCredential::new::<RemoteEcdsaKey<S, PEK, APC>>(key_id, wte, trust_anchors)?;
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
    use nl_wallet_mdoc::holder::TrustAnchor;
    use openid4vc::jwt::JwtCredential;
    use platform_support::hw_keystore::PlatformEcdsaKey;
    use wallet_common::{
        keys::{software::SoftwareEcdsaKey, StoredByIdentifier},
        utils::random_string,
    };

    use crate::{
        account_provider::AccountProviderClient, instruction::InstructionClient, storage::Storage,
        wallet::PidIssuanceError,
    };

    use super::WteIssuanceClient;

    pub struct MockWteIssuanceClient;

    impl WteIssuanceClient for MockWteIssuanceClient {
        async fn obtain_wte<S, PEK, APC>(
            &self,
            _trust_anchors: &[TrustAnchor<'_>],
            _remote_instruction: &InstructionClient<'_, S, PEK, APC>,
        ) -> Result<JwtCredential, PidIssuanceError>
        where
            S: Storage,
            PEK: PlatformEcdsaKey,
            APC: AccountProviderClient,
        {
            let key_id = random_string(32);
            SoftwareEcdsaKey::new_unique(&key_id).unwrap();
            let cred = JwtCredential::new_unverified::<SoftwareEcdsaKey>(key_id, "header.body.signature".into());

            Ok(cred)
        }
    }

    impl Default for MockWteIssuanceClient {
        fn default() -> Self {
            Self
        }
    }
}
