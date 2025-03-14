use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use futures::future::try_join_all;
use nutype::nutype;
use sd_jwt::metadata::SpecOptionalImplRequired;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use jwt::credential::JwtCredentialClaims;
use jwt::jwk::jwk_jwt_header;
use jwt::pop::JwtPopClaims;
use jwt::Jwt;
use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::utils::serialization::CborBase64;
use nl_wallet_mdoc::IssuerSigned;
use poa::Poa;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::urls::BaseUrl;
use wallet_common::vec_at_least::VecNonEmpty;
use wallet_common::wte::WteClaims;

use crate::credential_formats::CredentialFormat;
use crate::credential_formats::CredentialType;
use crate::issuance_session::IssuanceSessionError;
use crate::Format;

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-8.1>.
/// Sent JSON-encoded to `POST /batch_credential`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequests {
    pub credential_requests: VecNonEmpty<CredentialRequest>,
    pub attestations: Option<WteDisclosure>,
    pub poa: Option<Poa>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WteDisclosure(
    pub(crate) Jwt<JwtCredentialClaims<WteClaims>>,
    pub(crate) Jwt<JwtPopClaims>,
);

impl WteDisclosure {
    pub fn new(wte: Jwt<JwtCredentialClaims<WteClaims>>, release: Jwt<JwtPopClaims>) -> Self {
        Self(wte, release)
    }
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-7.2>.
/// Sent JSON-encoded to `POST /credential`.
// TODO: add `wallet_attestation`, `wallet_attestation_pop`, and `proof_of_secure_combination` (PVW-2361, PVW-2362)
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequest {
    #[serde(flatten)]
    pub credential_type: SpecOptionalImplRequired<CredentialRequestType>,
    pub proof: Option<CredentialRequestProof>,
    pub attestations: Option<WteDisclosure>,
    pub poa: Option<Poa>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialRequestType {
    MsoMdoc { doctype: String },
}

impl Display for CredentialRequestType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialRequestType::MsoMdoc { doctype } => write!(f, "MsoMdoc({doctype})"),
        }
    }
}

impl CredentialRequestType {
    pub fn matches<T: CredentialType>(&self, other: &T) -> bool {
        self.format() == other.format() && self.credential_type() == other.credential_type()
    }
}

impl CredentialFormat for CredentialRequestType {
    fn format(&self) -> Format {
        match self {
            CredentialRequestType::MsoMdoc { .. } => Format::MsoMdoc,
        }
    }
}

impl CredentialType for CredentialRequestType {
    fn credential_type(&self) -> String {
        match self {
            CredentialRequestType::MsoMdoc { doctype } => doctype.clone(),
        }
    }
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-endpoint>
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "proof_type", rename_all = "snake_case")]
pub enum CredentialRequestProof {
    Jwt { jwt: Jwt<JwtPopClaims> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialResponses {
    pub credential_responses: Vec<CredentialResponse>,
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-response>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialResponse {
    MsoMdoc { credential: Box<CborBase64<IssuerSigned>> },
}

impl CredentialFormat for CredentialResponse {
    fn format(&self) -> Format {
        match self {
            CredentialResponse::MsoMdoc { .. } => Format::MsoMdoc,
        }
    }
}

pub const OPENID4VCI_VC_POP_JWT_TYPE: &str = "openid4vci-proof+jwt";

impl CredentialRequestProof {
    pub async fn new_multiple<K: CredentialEcdsaKey>(
        nonce: String,
        wallet_client_id: String,
        credential_issuer_identifier: BaseUrl,
        number_of_keys: u64,
        key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<Vec<(K, CredentialRequestProof)>, IssuanceSessionError> {
        let keys = key_factory
            .generate_new_multiple(number_of_keys)
            .await
            .map_err(|e| IssuanceSessionError::PrivateKeyGeneration(Box::new(e)))?;

        let payload = JwtPopClaims::new(
            Some(nonce),
            wallet_client_id,
            credential_issuer_identifier.as_ref().to_string(),
        );

        let keys_and_jwt_payloads = try_join_all(keys.into_iter().map(|privkey| async {
            let header = jwk_jwt_header(OPENID4VCI_VC_POP_JWT_TYPE, &privkey).await?;
            let payload = payload.clone();
            Ok::<_, IssuanceSessionError>((privkey, (payload, header)))
        }))
        .await?;

        let keys_and_proofs = Jwt::sign_bulk(keys_and_jwt_payloads, key_factory)
            .await?
            .into_iter()
            .map(|(key, jwt)| (key, CredentialRequestProof::Jwt { jwt }))
            .collect();

        Ok(keys_and_proofs)
    }
}

/// Stores multiple copies of credentials that have identical attributes.
#[nutype(
    validate(predicate = |copies| !copies.is_empty()),
    derive(Debug, Clone, AsRef, TryFrom, Serialize, Deserialize, PartialEq, IntoIterator)
)]
pub struct CredentialCopies<T>(Vec<T>);

pub type MdocCopies = CredentialCopies<Mdoc>;

impl<T> CredentialCopies<T> {
    pub fn first(&self) -> &T {
        self.as_ref().first().unwrap()
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}
