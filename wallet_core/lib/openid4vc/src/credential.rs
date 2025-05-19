use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use futures::future::try_join_all;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::TryFromInto;

use crypto::factory::KeyFactory;
use crypto::keys::CredentialEcdsaKey;
use http_utils::urls::BaseUrl;
use jwt::credential::JwtCredentialClaims;
use jwt::jwk::jwk_jwt_header;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteClaims;
use jwt::Jwt;
use mdoc::holder::Mdoc;
use mdoc::utils::serialization::CborBase64;
use mdoc::IssuerSigned;
use poa::Poa;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;

use crate::issuance_session::IssuanceSessionError;
use crate::token::AuthorizationCode;
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
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequest {
    #[serde(flatten)]
    pub credential_type: SpecOptional<CredentialRequestType>,
    pub proof: Option<CredentialRequestProof>,
    pub attestations: Option<WteDisclosure>,
    pub poa: Option<Poa>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialRequestType {
    MsoMdoc {
        doctype: String,
    },

    #[serde(rename = "dc+sd-jwt")]
    SdJwt {
        vct: String,
    },
}

impl CredentialRequestType {
    pub fn format(&self) -> Format {
        match self {
            CredentialRequestType::MsoMdoc { .. } => Format::MsoMdoc,
            CredentialRequestType::SdJwt { .. } => Format::SdJwt,
        }
    }
}

impl Display for CredentialRequestType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialRequestType::MsoMdoc { doctype } => write!(f, "MsoMdoc({doctype})"),
            CredentialRequestType::SdJwt { vct } => write!(f, "SdJwt({vct})"),
        }
    }
}

impl CredentialRequestType {
    pub fn from_format(format: Format, attestation_type: String) -> Option<Self> {
        match format {
            Format::MsoMdoc => Some(CredentialRequestType::MsoMdoc {
                doctype: attestation_type,
            }),
            Format::SdJwt => Some(CredentialRequestType::SdJwt { vct: attestation_type }),
            _ => None,
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
    SdJwt { credential: String },
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

pub const OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME: &str = "openid-credential-offer";

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOffer {
    pub credential_issuer: BaseUrl,
    pub credential_configuration_ids: Vec<String>,
    pub grants: Option<Grants>,
}

/// OpenID4VCI protocol message containing the credential offer.
/// The Credential Offer is passed as a single URI-encoded parameter containing a JSON-encoded value.
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-offer>
/// Note: the spec says that this may contain a `credential_offer_uri` instead of a `credential_offer`, but we don't
/// support that (yet).
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOfferContainer {
    #[serde_as(as = "TryFromInto<String>")]
    pub credential_offer: CredentialOffer,
}

impl TryFrom<String> for CredentialOffer {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

impl TryInto<String> for CredentialOffer {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
    }
}

/// Grants for a Verifiable Credential.
/// May contain either or both. If it contains both, it is up to the wallet which one it uses.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Grants {
    Both {
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
        authorization_code: GrantAuthorizationCode,
    },
    AuthorizationCode {
        authorization_code: GrantAuthorizationCode,
    },
    PreAuthorizedCode {
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
    },
}

impl Grants {
    pub fn authorization_code(&self) -> Option<AuthorizationCode> {
        match self {
            Grants::Both {
                pre_authorized_code, ..
            } => Some(pre_authorized_code.pre_authorized_code.clone()),
            Grants::PreAuthorizedCode { pre_authorized_code } => Some(pre_authorized_code.pre_authorized_code.clone()),
            Grants::AuthorizationCode { .. } => None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantAuthorizationCode {
    pub issuer_state: Option<String>,
    pub authorization_server: Option<BaseUrl>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantPreAuthorizedCode {
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: AuthorizationCode,
    pub tx_code: Option<PreAuthTransactionCode>,
    pub authorization_server: Option<BaseUrl>,
}

impl GrantPreAuthorizedCode {
    pub fn new(pre_authorized_code: AuthorizationCode) -> Self {
        Self {
            pre_authorized_code,
            tx_code: None,
            authorization_server: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreAuthTransactionCode {
    pub input_mode: Option<String>,
    pub length: Option<u64>,
    pub description: Option<String>,
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

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::json;

    use crate::credential::Grants;

    #[test]
    fn test_grants_serialization() {
        let json = json!({
            "authorization_code": { "issuer_state": "foo" },
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        assert_matches!(serde_json::from_value::<Grants>(json).unwrap(), Grants::Both { .. });

        let json = json!({
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        assert_matches!(
            serde_json::from_value::<Grants>(json).unwrap(),
            Grants::PreAuthorizedCode { .. }
        );

        let json = json!({
            "authorization_code": { "issuer_state": "foo" }
        });
        assert_matches!(
            serde_json::from_value::<Grants>(json).unwrap(),
            Grants::AuthorizationCode { .. }
        );
    }
}
