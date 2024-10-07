use std::collections::HashMap;

use nutype::nutype;
use ring::hmac;
use serde::Deserialize;
use serde_with::{hex::Hex, serde_as};

use nl_wallet_mdoc::{
    holder::TrustAnchor,
    utils::x509::{CertificateType, CertificateUsage},
};
use openid4vc::verifier::{SessionTypeReturnUrl, UseCase, UseCases};
use wallet_common::{generator::TimeGenerator, trust_anchor::DerTrustAnchor, urls::CorsOrigin};

use super::*;

const MIN_KEY_LENGTH_BYTES: usize = 16;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Verifier {
    pub reader_trust_anchors: Option<Vec<Certificate>>,
    pub usecases: VerifierUseCases,
    #[serde(alias = "trust_anchors")] // TODO: introduced for backwards compatibility, remove alias after notifying RPs
    pub issuer_trust_anchors: Vec<DerTrustAnchor>,
    #[serde_as(as = "Hex")]
    pub ephemeral_id_secret: EhpemeralIdSecret,
    pub allow_origins: Option<CorsOrigin>,
}

impl Verifier {
    pub fn verify_all<'a>(&'a self) -> Result<(), CertificateVerificationError> {
        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'a>> = self
            .reader_trust_anchors
            .iter()
            .flatten()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CertificateVerificationError::InvalidTrustAnchor)?;

        let certificates: Vec<(String, Certificate)> = self
            .usecases
            .iter()
            .map(|(use_case_id, usecase)| (use_case_id.clone(), usecase.key_pair.certificate.clone().into()))
            .collect();

        verify_certificates(
            &certificates,
            &trust_anchors,
            CertificateUsage::ReaderAuth,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::ReaderAuth(Some(_))),
        )
    }
}

#[nutype(derive(Clone, From, Deserialize, Deref, AsRef))]
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
    type Error = anyhow::Error;

    fn try_from(value: &VerifierUseCase) -> Result<Self, Self::Error> {
        let use_case = UseCase::try_new((&value.key_pair).try_into()?, value.session_type_return_url)?;

        Ok(use_case)
    }
}

impl From<&EhpemeralIdSecret> for hmac::Key {
    fn from(value: &EhpemeralIdSecret) -> Self {
        hmac::Key::new(hmac::HMAC_SHA256, value.as_ref())
    }
}
