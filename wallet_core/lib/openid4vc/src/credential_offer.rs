use http_utils::urls::BaseUrl;
use serde::Deserialize;
use serde::Serialize;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::token::AuthorizationCode;

pub const OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME: &str = "openid-credential-offer";

/// OpenID4VCI protocol message containing the credential offer.
/// The Credential Offer is passed as a single URI-encoded parameter containing a JSON-encoded value.
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-offer>
/// Note: the spec says that this may contain a `credential_offer_uri` instead of a `credential_offer`, but we don't
/// support that (yet).
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOfferContainer {
    #[serde_as(as = "JsonString")]
    pub credential_offer: CredentialOffer,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOffer {
    pub credential_issuer: IssuerIdentifier,
    pub credential_configuration_ids: Vec<CredentialConfigurationId>,
    pub grants: Option<Grants>,
}

impl CredentialOffer {
    pub fn pre_authorized_code(&self) -> Option<&AuthorizationCode> {
        self.grants.as_ref()?.pre_authorized_code()
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
    pub fn pre_authorized_code(&self) -> Option<&AuthorizationCode> {
        match self {
            Grants::Both {
                pre_authorized_code, ..
            } => Some(&pre_authorized_code.pre_authorized_code),
            Grants::PreAuthorizedCode { pre_authorized_code } => Some(&pre_authorized_code.pre_authorized_code),
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

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::json;

    use super::Grants;

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
