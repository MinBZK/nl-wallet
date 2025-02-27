use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;

use chrono::Days;
use futures::future::join_all;
use indexmap::IndexMap;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;
use serde_with::base64::Base64;
use serde_with::serde_as;

use openid4vc_server::issuer::IssuerKeyRing;
use sd_jwt::metadata::TypeMetadata;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::p256_der::DerVerifyingKey;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::HttpsUri;
use wallet_common::utils;

use crate::pid::attributes::BrpPidAttributeService;
use crate::pid::attributes::Error as BrpError;
use crate::pid::brp::client::HttpBrpClient;

use super::*;

#[serde_as]
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

    #[serde_as(as = "Base64")]
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
    pub fn issuer_uris(&self) -> Result<IndexMap<String, HttpsUri>, BrpError> {
        self.private_keys
            .iter()
            .map(|(doctype, key_pair)| {
                let issuer_san_dns_name_or_uris = key_pair.certificate.san_dns_name_or_uris()?;
                let issuer_uri = match issuer_san_dns_name_or_uris.len() {
                    NonZeroUsize::MIN => Ok(issuer_san_dns_name_or_uris.into_first()),
                    n => Err(BrpError::UnexpectedIssuerSanDnsNameOrUrisCount(n)),
                }?;

                Ok((doctype.to_owned(), issuer_uri))
            })
            .collect::<Result<IndexMap<_, _>, _>>()
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
            issuer.issuer_uris()?,
            issuer.metadata(),
            Days::new(issuer.valid_days),
            issuer.copy_count,
        )
    }
}

impl TryFromKeySettings<HashMap<String, KeyPair>> for IssuerKeyRing<PrivateKeyVariant> {
    type Error = PrivateKeySettingsError;

    async fn try_from_key_settings(
        private_keys: HashMap<String, KeyPair>,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<Self, Self::Error> {
        let iter = private_keys.into_iter().map(|(doctype, key_pair)| async {
            let result = (
                doctype,
                ParsedKeyPair::try_from_key_settings(key_pair, hsm.clone()).await?,
            );
            Ok(result)
        });

        let issuer_keys = join_all(iter)
            .await
            .into_iter()
            .collect::<Result<HashMap<String, ParsedKeyPair<PrivateKeyVariant>>, Self::Error>>()?;

        Ok(issuer_keys.into())
    }
}
