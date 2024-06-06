use std::collections::HashMap;

use nutype::nutype;
use ring::hmac;
use serde::Deserialize;
use serde_with::hex::Hex;
use serde_with::serde_as;

use nl_wallet_mdoc::verifier::{SessionTypeReturnUrl, UseCase, UseCases};
use wallet_common::trust_anchor::DerTrustAnchor;

use super::*;

const MIN_KEY_LENGTH_BYTES: usize = 16;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Verifier {
    pub usecases: VerifierUseCases,
    pub trust_anchors: Vec<DerTrustAnchor>,
    #[serde_as(as = "Hex")]
    pub ephemeral_id_secret: EhpemeralIdSecret,
}

#[nutype(derive(Clone, Deserialize))]
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
    type Error = p256::pkcs8::Error;

    fn try_from(value: VerifierUseCases) -> Result<Self, Self::Error> {
        let use_cases = value
            .into_inner()
            .into_iter()
            .map(|(id, use_case)| {
                let use_case = UseCase::try_from(&use_case)?;

                Ok((id, use_case))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?
            .into();

        Ok(use_cases)
    }
}

impl TryFrom<&VerifierUseCase> for UseCase {
    type Error = p256::pkcs8::Error;

    fn try_from(value: &VerifierUseCase) -> Result<Self, Self::Error> {
        let use_case = UseCase {
            key_pair: (&value.key_pair).try_into()?,
            session_type_return_url: value.session_type_return_url,
        };

        Ok(use_case)
    }
}

impl From<&EhpemeralIdSecret> for hmac::Key {
    fn from(value: &EhpemeralIdSecret) -> Self {
        hmac::Key::new(hmac::HMAC_SHA256, value.as_ref())
    }
}
