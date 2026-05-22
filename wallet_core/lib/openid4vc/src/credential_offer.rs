use http_utils::urls::BaseUrl;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::json::JsonString;
use serde_with::rust::deserialize_ignore_any;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use strum::EnumString;
use utils::vec_at_least::VecNonEmpty;

use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::token::AuthorizationCode;

pub const OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME: &str = "openid-credential-offer";

/// OpenID4VCI protocol message containing the credential offer. The Credential Offer contains a single URI query
/// parameter, either `credential_offer` or `credential_offer_uri`.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1>
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialOfferContainer {
    Offer {
        #[serde_as(as = "JsonString")]
        credential_offer: Box<CredentialOffer>,
    },

    Uri {
        credential_offer_uri: BaseUrl,
    },
}

impl CredentialOfferContainer {
    pub fn new_offer(credential_offer: CredentialOffer) -> Self {
        Self::Offer {
            credential_offer: Box::new(credential_offer),
        }
    }

    pub fn new_uri(credential_offer_uri: BaseUrl) -> Self {
        Self::Uri { credential_offer_uri }
    }
}

/// An OpenID4VCI Credential Offer, which is the starting point for issuance. It contains information needed for the
/// Authorization Code flow, the Pre-Authorized Code Flow or both.
///
/// https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialOffer {
    /// The URL of the Credential Issuer, as defined in Section 12.2.1, from which the Wallet is requested to obtain
    /// one or more Credentials. The Wallet uses it to obtain the Credential Issuer's Metadata following the steps
    /// defined in Section 12.2.2.
    pub credential_issuer: IssuerIdentifier,

    /// A non-empty array of unique strings that each identify one of the keys in the name/value pairs stored in the
    /// credential_configurations_supported Credential Issuer metadata. The Wallet uses these string values to obtain
    /// the respective object that contains information about the Credential being offered as defined in Section
    /// 12.2.4. For example, these string values can be used to obtain scope values to be used in the Authorization
    /// Request.
    pub credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,

    /// Object indicating to the Wallet the Grant Types the Credential Issuer's Authorization Server is prepared to
    /// process for this Credential Offer.
    #[serde(skip_serializing_if = "CredentialOffer::grants_is_none_or_other")]
    pub grants: Option<Grants>,
}

impl CredentialOffer {
    pub fn new_pre_authorized(
        credential_issuer: IssuerIdentifier,
        credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,
        pre_authorized_code: AuthorizationCode,
    ) -> Self {
        Self {
            credential_issuer,
            credential_configuration_ids,
            grants: Some(Grants::new_pre_authorized(pre_authorized_code)),
        }
    }

    fn grants_is_none_or_other(grants: &Option<Grants>) -> bool {
        grants
            .as_ref()
            .map(|grants| matches!(grants, Grants::Other))
            .unwrap_or(true)
    }
}

/// Object indicating to the Wallet the Grant Types the Credential Issuer's Authorization Server is prepared to process
/// for this Credential Offer. Every grant is represented by a name/value pair. The name is the Grant Type identifier;
/// the value is an object that contains parameters either determining the way the Wallet MUST use the particular grant
/// and/or parameters the Wallet MUST send with the respective request(s). If grants is not present or is empty, the
/// Wallet MUST determine the Grant Types the Credential Issuer's Authorization Server supports using the respective
/// metadata. When multiple grants are present, it is at the Wallet's discretion which one to use.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-4>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Grants {
    Both {
        authorization_code: GrantAuthorizationCode,
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
    },
    AuthorizationCode {
        authorization_code: GrantAuthorizationCode,
    },
    PreAuthorizedCode {
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
    },
    #[serde(deserialize_with = "deserialize_ignore_any")]
    Other,
}

impl Grants {
    pub fn new_pre_authorized(pre_authorized_code: AuthorizationCode) -> Self {
        Self::PreAuthorizedCode {
            pre_authorized_code: GrantPreAuthorizedCode::new(pre_authorized_code),
        }
    }
}

/// Grant Type `authorization_code`.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-5.1.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrantAuthorizationCode {
    /// String value created by the Credential Issuer and opaque to the Wallet that is used to bind the subsequent
    /// Authorization Request with a context set up during previous process steps. If the Wallet decides to use the
    /// Authorization Code Flow and received a value for this parameter, it MUST include it in the subsequent
    /// Authorization Request to the Authorization Server as the `issuer_state` parameter value.
    pub issuer_state: Option<String>,

    /// Optional string that the Wallet can use to identify the Authorization Server to use with this grant type when
    /// authorization_servers parameter in the Credential Issuer metadata has multiple entries. It MUST NOT be used
    /// otherwise. The value of this parameter MUST match with one of the values in the `authorization_servers` array
    /// obtained from the Credential Issuer metadata.
    pub authorization_server: Option<IssuerIdentifier>,
}

/// Grant Type `urn:ietf:params:oauth:grant-type:pre-authorized_code`.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-5.2.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPreAuthorizedCode {
    /// The code representing the Credential Issuer's authorization for the Wallet to obtain Credentials of a certain
    /// type. This code MUST be short lived and single use. If the Wallet decides to use the Pre-Authorized Code Flow,
    /// this parameter value MUST be included in the subsequent Token Request with the Pre-Authorized Code Flow.
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: AuthorizationCode,

    /// Object indicating that a Transaction Code is required if present, even if empty.
    pub tx_code: Option<PreAuthTransactionCode>,

    /// Optional string that the Wallet can use to identify the Authorization Server to use with this grant type when
    /// authorization_servers parameter in the Credential Issuer metadata has multiple entries. It MUST NOT be used
    /// otherwise. The value of this parameter MUST match with one of the values in the `authorization_servers` array
    /// obtained from the Credential Issuer metadata.
    pub authorization_server: Option<IssuerIdentifier>,
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

/// Object indicating that a Transaction Code is required if present, even if empty. It describes the requirements for a
/// Transaction Code, which the Authorization Server expects the End-User to present along with the Token Request in a
/// Pre-Authorized Code Flow. If the Authorization Server does not expect a Transaction Code, this object is absent;
/// this is the default. The Transaction Code is intended to bind the Pre-Authorized Code to a certain transaction to
/// prevent replay of this code by an attacker that, for example, scanned the QR code while standing behind the
/// legitimate End-User. It is RECOMMENDED to send the Transaction Code via a separate channel. If the Wallet decides to
/// use the Pre-Authorized Code Flow, the Transaction Code value MUST be sent in the tx_code parameter with the
/// respective Token Request as defined in Section 6.1. If no `length`, `description`, or `input_mode` is given, this
/// object MAY be empty.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-5.2.2.2.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreAuthTransactionCode {
    /// String specifying the input character set.
    pub input_mode: Option<PreAuthTransactionCodeInputMode>,

    /// Integer specifying the length of the Transaction Code. This helps the Wallet to render the input screen and
    /// improve the user experience.
    pub length: Option<u8>,

    /// String containing guidance for the Holder of the Wallet on how to obtain the Transaction Code, e.g., describing
    /// over which communication channel it is delivered. The Wallet is RECOMMENDED to display this description next to
    /// the Transaction Code input screen to improve the user experience. The length of the string MUST NOT exceed 300
    /// characters. The description does not support internationalization, however the Issuer MAY detect the Holder's
    /// language by previous communication or an HTTP Accept-Language header within an HTTP GET request for a
    /// Credential Offer URI.
    pub description: Option<String>,
}

/// String specifying the input character set. Possible values are `numeric` (only digits) and `text` (any characters).
/// The default is `numeric`.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-5.2.2.2.2.1>
#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr)]
#[strum(serialize_all = "lowercase")]
pub enum PreAuthTransactionCodeInputMode {
    #[default]
    Numeric,
    Text,
    #[strum(default)]
    Other(String),
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::json;

    use super::CredentialOffer;
    use super::Grants;
    use super::PreAuthTransactionCodeInputMode;

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

        let json = json!({});
        assert_matches!(serde_json::from_value::<Grants>(json).unwrap(), Grants::Other);

        let json = json!({
            "foo": "bar"
        });
        assert_matches!(serde_json::from_value::<Grants>(json).unwrap(), Grants::Other);
    }

    #[test]
    fn test_credential_offer_serialization() {
        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-6
        let credential_offer_json = json!({
            "credential_issuer": "https://credential-issuer.example.com",
            "credential_configuration_ids": [
                "UniversityDegreeCredential",
                "org.iso.18013.5.1.mDL"
            ],
            "grants": {
                "urn:ietf:params:oauth:grant-type:pre-authorized_code": {
                    "pre-authorized_code": "oaKazRN8I0IbtZ0C7JuMn5",
                    "tx_code": {
                        "length": 4,
                        "input_mode": "numeric",
                        "description": "Please provide the one-time code that was sent via e-mail"
                    }
                }
            }
        });

        let credential_offer = serde_json::from_value::<CredentialOffer>(credential_offer_json.clone())
            .expect("should be able to deserialize CredentialOffer");

        assert_eq!(
            credential_offer.credential_issuer.as_ref(),
            "https://credential-issuer.example.com"
        );
        assert!(
            credential_offer
                .credential_configuration_ids
                .iter()
                .map(AsRef::as_ref)
                .eq(["UniversityDegreeCredential", "org.iso.18013.5.1.mDL"])
        );

        let grant_pre_auth = match &credential_offer.grants {
            Some(Grants::PreAuthorizedCode { pre_authorized_code }) => pre_authorized_code,
            _ => panic!("JSON should contain Pre-Authorized Code grant"),
        };

        assert_eq!(grant_pre_auth.pre_authorized_code.as_ref(), "oaKazRN8I0IbtZ0C7JuMn5");

        assert_eq!(
            grant_pre_auth
                .tx_code
                .as_ref()
                .and_then(|tx_code| tx_code.input_mode.as_ref()),
            Some(&PreAuthTransactionCodeInputMode::Numeric)
        );
        assert_eq!(
            grant_pre_auth.tx_code.as_ref().and_then(|tx_code| tx_code.length),
            Some(4)
        );
        assert_eq!(
            grant_pre_auth
                .tx_code
                .as_ref()
                .and_then(|tx_code| tx_code.description.as_deref()),
            Some("Please provide the one-time code that was sent via e-mail")
        );

        assert_eq!(serde_json::to_value(credential_offer).unwrap(), credential_offer_json);
    }
}
