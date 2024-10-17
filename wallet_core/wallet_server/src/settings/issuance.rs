use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

use nl_wallet_mdoc::utils::x509::Certificate;
use wallet_common::{reqwest::deserialize_certificates, urls::BaseUrl};

use super::*;
use crate::pid::{
    attributes::{BrpPidAttributeService, Error as BrpError},
    brp::client::HttpBrpClient,
};

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Issuer {
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

    #[serde_as(as = "Base64")]
    pub wte_issuer_pubkey: Vec<u8>,
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
