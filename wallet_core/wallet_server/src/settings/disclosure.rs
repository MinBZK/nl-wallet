use std::collections::HashMap;

use derive_more::AsRef;
use derive_more::From;
use derive_more::IntoIterator;
use futures::future::join_all;
use nutype::nutype;
use ring::hmac;
use serde::Deserialize;
use serde_with::hex::Hex;
use serde_with::serde_as;

use nl_wallet_mdoc::server_keys;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::UseCases;
use wallet_common::urls::CorsOrigin;

use crate::keys::PrivateKeyType;

use super::*;

const MIN_KEY_LENGTH_BYTES: usize = 16;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Verifier {
    pub usecases: VerifierUseCases,
    #[serde_as(as = "Hex")]
    pub ephemeral_id_secret: EhpemeralIdSecret,
    pub allow_origins: Option<CorsOrigin>,
}

#[derive(Clone, From, AsRef, IntoIterator, Deserialize)]
pub struct VerifierUseCases(HashMap<String, VerifierUseCase>);

#[nutype(validate(predicate = |v| v.len() >= MIN_KEY_LENGTH_BYTES), derive(Clone, TryFrom, AsRef, Deserialize))]
pub struct EhpemeralIdSecret(Vec<u8>);

#[derive(Clone, Deserialize)]
pub struct VerifierUseCase {
    #[serde(default)]
    pub session_type_return_url: SessionTypeReturnUrl,
    #[serde(flatten)]
    pub key_pair: KeyPair,
}

impl TryFromKeySettings<VerifierUseCases> for UseCases<PrivateKeyType> {
    type Error = anyhow::Error;

    async fn try_from_key_settings(value: VerifierUseCases, hsm: Option<&Pkcs11Hsm>) -> Result<Self, Self::Error> {
        let iter = value.into_iter().map(|(id, use_case)| async move {
            let result = (id, UseCase::try_from_key_settings(use_case, hsm).await?);
            Ok(result)
        });

        let use_cases = join_all(iter)
            .await
            .into_iter()
            .collect::<Result<HashMap<String, UseCase<_>>, Self::Error>>()?;

        Ok(use_cases.into())
    }
}

impl TryFromKeySettings<VerifierUseCase> for UseCase<PrivateKeyType> {
    type Error = anyhow::Error;

    async fn try_from_key_settings(value: VerifierUseCase, hsm: Option<&Pkcs11Hsm>) -> Result<Self, Self::Error> {
        let use_case = UseCase::try_new(
            server_keys::KeyPair::try_from_key_settings(value.key_pair, hsm).await?,
            value.session_type_return_url,
        )?;

        Ok(use_case)
    }
}

impl From<&EhpemeralIdSecret> for hmac::Key {
    fn from(value: &EhpemeralIdSecret) -> Self {
        hmac::Key::new(hmac::HMAC_SHA256, value.as_ref())
    }
}
