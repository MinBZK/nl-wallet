use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;

use chrono::Days;
use indexmap::IndexMap;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;

use sd_jwt::metadata::TypeMetadata;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::urls::BaseUrl;
use wallet_common::utils;

use crate::pid::attributes::BrpPidAttributeService;
use crate::pid::attributes::Error as BrpError;
use crate::pid::brp::client::HttpBrpClient;

use super::*;

#[derive(Clone, Deserialize)]
pub struct Issuer {
    /// Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    #[serde(deserialize_with = "deserialize_type_metadata")]
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

    pub valid_days: u64,

    pub copy_count: NonZeroU8,
}

fn deserialize_type_metadata<'de, D>(deserializer: D) -> Result<Vec<TypeMetadata>, D::Error>
where
    D: Deserializer<'de>,
{
    let path = Vec::<String>::deserialize(deserializer)?;

    let metadatas = path
        .iter()
        .map(|path| {
            let metadata_file = fs::read(utils::prefix_local_path(path.as_ref())).map_err(de::Error::custom)?;
            serde_json::from_slice(metadata_file.as_slice())
        })
        .collect::<Result<_, _>>()
        .map_err(de::Error::custom)?;

    Ok(metadatas)
}

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub bsn_privkey: String,
    pub http_config: TlsPinningConfig,
}

impl Issuer {
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
            issuer.metadata(),
            Days::new(issuer.valid_days),
            issuer.copy_count,
        )
    }
}
