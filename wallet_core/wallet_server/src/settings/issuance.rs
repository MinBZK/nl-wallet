use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::{
    holder::TrustAnchor,
    utils::x509::{Certificate, CertificateType, CertificateUsage},
};
use wallet_common::{generator::TimeGenerator, reqwest::deserialize_certificates, urls::BaseUrl};

use super::*;
use crate::pid::{
    attributes::{BrpPidAttributeService, Error as BrpError},
    brp::client::HttpBrpClient,
};

#[derive(Clone, Deserialize)]
pub struct Issuer {
    #[cfg(not(feature = "disclosure"))]
    pub issuer_trust_anchors: Option<Vec<Certificate>>,

    // Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: Vec<String>,

    pub digid: Digid,

    pub brp_server: BaseUrl,
}

impl Issuer {
    pub fn verify_all<'a>(&'a self, trust_anchors: &[TrustAnchor<'a>]) -> Result<(), CertificateVerificationError> {
        let time = TimeGenerator;

        let certificates: Vec<(String, Certificate)> = self
            .private_keys
            .iter()
            .map(|(id, keypair)| (id.clone(), Certificate::from(keypair.certificate.clone())))
            .collect();

        verify_certificates(
            &certificates,
            trust_anchors,
            CertificateUsage::Mdl,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::Mdl(Some(_))),
        )
    }
}

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub issuer_url: BaseUrl,
    pub bsn_privkey: String,
    #[serde(deserialize_with = "deserialize_certificates", default)]
    pub trust_anchors: Vec<reqwest::Certificate>,
}

impl Issuer {
    pub fn certificates(&self) -> IndexMap<String, Certificate> {
        self.private_keys
            .iter()
            .map(|(doctype, privkey)| (doctype.clone(), privkey.certificate.clone().into()))
            .collect()
    }
}

impl TryFrom<&Issuer> for BrpPidAttributeService {
    type Error = BrpError;

    fn try_from(issuer: &Issuer) -> Result<Self, Self::Error> {
        BrpPidAttributeService::new(
            HttpBrpClient::new(issuer.brp_server.clone()),
            issuer.digid.issuer_url.clone(),
            &issuer.digid.bsn_privkey,
            issuer.digid.trust_anchors.clone(),
            issuer.certificates(),
        )
    }
}
