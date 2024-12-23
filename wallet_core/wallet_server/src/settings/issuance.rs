use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::base64::Base64;

use sd_jwt::metadata::TypeMetadata;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::urls::BaseUrl;

use crate::pid::attributes::BrpPidAttributeService;
use crate::pid::attributes::Error as BrpError;
use crate::pid::brp::client::HttpBrpClient;

use super::*;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Issuer {
    /// Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    #[serde_as(as = "Vec<Base64>")]
    pub metadata: Vec<TypeMetadata>,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: Vec<String>,

    pub digid: Digid,

    pub brp_server: BaseUrl,

    pub wte_issuer_pubkey: DerVerifyingKey,
}

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub bsn_privkey: String,
    pub http_config: TlsPinningConfig,
}

impl Issuer {
    pub fn certificates(&self) -> IndexMap<String, BorrowingCertificate> {
        self.private_keys
            .iter()
            .map(|(doctype, privkey)| (doctype.clone(), privkey.certificate.clone()))
            .collect()
    }

    pub fn metadata(&self) -> IndexMap<String, TypeMetadata> {
        self.metadata
            .iter()
            .map(|type_metadata| (type_metadata.vct.clone(), type_metadata.clone()))
            .collect()
    }
}

impl TryFrom<&Issuer> for BrpPidAttributeService {
    type Error = BrpError;

    fn try_from(issuer: &Issuer) -> Result<Self, Self::Error> {
        BrpPidAttributeService::new(
            HttpBrpClient::new(issuer.brp_server.clone()),
            &issuer.digid.bsn_privkey,
            issuer.digid.http_config.clone(),
            issuer.certificates(),
            issuer.metadata(),
        )
    }
}
