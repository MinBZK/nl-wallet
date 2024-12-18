use futures::future::try_join_all;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::utils::serialization::CborBase64;
use nl_wallet_mdoc::IssuerSigned;
use wallet_common::jwt::jwk_jwt_header;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtCredentialClaims;
use wallet_common::jwt::JwtPopClaims;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::poa::Poa;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::nonempty::NonEmpty;
use wallet_common::urls::BaseUrl;
use wallet_common::wte::WteClaims;

use crate::issuance_session::IssuanceSessionError;
use crate::token::CredentialPreview;
use crate::Format;

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-8.1>.
/// Sent JSON-encoded to `POST /batch_credential`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequests {
    pub credential_requests: NonEmpty<Vec<CredentialRequest>>,
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
    pub credential_type: CredentialRequestType,
    pub proof: Option<CredentialRequestProof>,
    pub attestations: Option<WteDisclosure>,
    pub poa: Option<Poa>,
}

impl CredentialRequest {
    pub fn credential_type(&self) -> Option<&str> {
        match &self.credential_type {
            CredentialRequestType::MsoMdoc { doctype } => doctype.as_ref().map(String::as_str),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialRequestType {
    MsoMdoc { doctype: Option<String> },
}

impl From<&CredentialPreview> for CredentialRequestType {
    fn from(value: &CredentialPreview) -> Self {
        match value {
            CredentialPreview::MsoMdoc { unsigned_mdoc, .. } => CredentialRequestType::MsoMdoc {
                doctype: Some(unsigned_mdoc.doc_type.clone()),
            },
        }
    }
}

impl From<&CredentialRequestType> for Format {
    fn from(value: &CredentialRequestType) -> Self {
        match value {
            CredentialRequestType::MsoMdoc { .. } => Format::MsoMdoc,
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

impl From<&CredentialResponse> for Format {
    fn from(value: &CredentialResponse) -> Self {
        match value {
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
    derive(Debug, Clone, AsRef, TryFrom, Serialize, Deserialize, PartialEq)
)]
pub struct CredentialCopies<T>(Vec<T>);

pub type MdocCopies = CredentialCopies<Mdoc>;

impl<T> IntoIterator for CredentialCopies<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

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
