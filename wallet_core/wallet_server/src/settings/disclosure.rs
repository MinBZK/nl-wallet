use std::collections::HashMap;

use derive_more::AsRef;
use derive_more::From;
use derive_more::IntoIterator;
use nutype::nutype;
use ring::hmac;
use serde::Deserialize;
use serde_with::hex::Hex;
use serde_with::serde_as;

use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::UseCases;
use wallet_common::urls::CorsOrigin;

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

impl TryFrom<VerifierUseCases> for UseCases {
    type Error = anyhow::Error;

    fn try_from(value: VerifierUseCases) -> Result<Self, Self::Error> {
        let use_cases = value
            .into_iter()
            .map(|(id, use_case)| {
                let use_case = UseCase::try_from(use_case)?;

                Ok((id, use_case))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?
            .into();

        Ok(use_cases)
    }
}

impl TryFrom<VerifierUseCase> for UseCase {
    type Error = anyhow::Error;

    fn try_from(value: VerifierUseCase) -> Result<Self, Self::Error> {
        let use_case = UseCase::try_new(value.key_pair.try_into_mdoc_key_pair()?, value.session_type_return_url)?;

        Ok(use_case)
    }
}

impl From<&EhpemeralIdSecret> for hmac::Key {
    fn from(value: &EhpemeralIdSecret) -> Self {
        hmac::Key::new(hmac::HMAC_SHA256, value.as_ref())
    }
}
