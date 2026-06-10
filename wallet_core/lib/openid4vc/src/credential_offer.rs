use std::collections::HashMap;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde::de::IgnoredAny;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use strum::EnumString;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::issuer_identifier::IssuerIdentifier;
use crate::issuer_identifier::IssuerUrl;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::token::AuthorizationCode;

pub const OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME: &str = "openid-credential-offer";

/// OpenID4VCI protocol message containing the credential offer. The Credential Offer contains a single URI query
/// parameter, either `credential_offer` or `credential_offer_uri`.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1>
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialOfferContainer {
    CredentialOffer(#[serde_as(as = "JsonString")] Box<CredentialOffer>),
    CredentialOfferUri(IssuerUrl),
}

impl CredentialOfferContainer {
    pub fn new_offer(credential_offer: CredentialOffer) -> Self {
        Self::CredentialOffer(Box::new(credential_offer))
    }

    pub fn new_uri(credential_offer_uri: IssuerUrl) -> Self {
        Self::CredentialOfferUri(credential_offer_uri)
    }

    /// Serialises this container as a URL query string (e.g. `credential_offer=%7B...%7D`), suitable for passing to
    /// [`Url::set_query`].
    pub fn to_query_string(&self) -> String {
        serde_qs::to_string(&self).expect("a `CredentialOffer` should always serialize")
    }

    /// Returns the key and value as a `(String, String)` pair, suitable for passing to
    /// [`form_urlencoded::Serializer::append_pair`] (e.g. as obtained from [`Url::query_pairs_mut`]).
    pub fn into_query_pair(self) -> (String, String) {
        let serde_json::Value::Object(map) = serde_json::to_value(&self).unwrap() else {
            unreachable!("`enum CredentialOfferContainer` should always serialize as an Object");
        };
        let (k, v) = map
            .into_iter()
            .exactly_one()
            .expect("`enum CredentialOfferContainer` should always serialize as an object with a single field");
        (k.to_string(), v.as_str().unwrap_or(&v.to_string()).to_string())
    }

    pub fn to_credential_offer_url(&self) -> Url {
        let mut url = Url::parse(&format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://")).expect("this is a valid URL");
        url.set_query(Some(&self.to_query_string()));
        url
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
    pub grants: Option<Grants>,
}

impl CredentialOffer {
    pub fn new_authorization(
        credential_issuer: IssuerIdentifier,
        credential_configuration_ids: VecNonEmpty<CredentialConfigurationId>,
        issuer_state: Option<String>,
    ) -> Self {
        Self {
            credential_issuer,
            credential_configuration_ids,
            grants: Some(Grants::new_authorization(issuer_state)),
        }
    }

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
}

/// Object indicating to the Wallet the Grant Types the Credential Issuer's Authorization Server is prepared to process
/// for this Credential Offer. Every grant is represented by a name/value pair. The name is the Grant Type identifier;
/// the value is an object that contains parameters either determining the way the Wallet MUST use the particular grant
/// and/or parameters the Wallet MUST send with the respective request(s). If grants is not present or is empty, the
/// Wallet MUST determine the Grant Types the Credential Issuer's Authorization Server supports using the respective
/// metadata. When multiple grants are present, it is at the Wallet's discretion which one to use.
///
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.1-4>
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Grants {
    pub authorization_code: Option<GrantAuthorizationCode>,

    #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
    pub pre_authorized_code: Option<GrantPreAuthorizedCode>,

    // Capture the keys of any unknown grant types.
    #[serde(flatten, skip_serializing)]
    pub unknown: HashMap<String, IgnoredAny>,
}

impl Grants {
    pub fn new_authorization(issuer_state: Option<String>) -> Self {
        Self {
            authorization_code: Some(GrantAuthorizationCode::new(issuer_state)),
            ..Self::default()
        }
    }

    pub fn new_pre_authorized(pre_authorized_code: AuthorizationCode) -> Self {
        Self {
            pre_authorized_code: Some(GrantPreAuthorizedCode::new(pre_authorized_code)),
            ..Self::default()
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

impl GrantAuthorizationCode {
    pub fn new(issuer_state: Option<String>) -> Self {
        Self {
            issuer_state,
            authorization_server: None,
        }
    }
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
    use serde_json::json;
    use url::Url;

    use super::CredentialOffer;
    use super::CredentialOfferContainer;
    use super::Grants;
    use super::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
    use super::PreAuthTransactionCodeInputMode;
    use crate::issuer_identifier::IssuerUrl;

    #[test]
    fn test_grants_serialization() {
        let json = json!({
            "authorization_code": { "issuer_state": "foo" },
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        let grants = serde_json::from_value::<Grants>(json).expect("should be able to deserialize Grants");

        assert!(grants.authorization_code.is_some());
        assert!(grants.pre_authorized_code.is_some());
        assert!(grants.unknown.is_empty());

        let json = json!({
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        let grants = serde_json::from_value::<Grants>(json).expect("should be able to deserialize Grants");

        assert!(grants.authorization_code.is_none());
        assert!(grants.pre_authorized_code.is_some());
        assert!(grants.unknown.is_empty());

        let json = json!({
            "authorization_code": { "issuer_state": "foo" }
        });
        let grants = serde_json::from_value::<Grants>(json).expect("should be able to deserialize Grants");

        assert!(grants.authorization_code.is_some());
        assert!(grants.pre_authorized_code.is_none());
        assert!(grants.unknown.is_empty());

        let json = json!({});
        let grants = serde_json::from_value::<Grants>(json).expect("should be able to deserialize Grants");

        assert!(grants.authorization_code.is_none());
        assert!(grants.pre_authorized_code.is_none());
        assert!(grants.unknown.is_empty());

        let json = json!({
            "foo": "bar"
        });
        let grants = serde_json::from_value::<Grants>(json).expect("should be able to deserialize Grants");

        assert!(grants.authorization_code.is_none());
        assert!(grants.pre_authorized_code.is_none());
        assert!(grants.unknown.keys().eq(["foo"]));
    }

    #[test]
    fn test_credential_offer_deserialization() {
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

        let grant_pre_auth = credential_offer
            .grants
            .as_ref()
            .and_then(|grants| grants.pre_authorized_code.as_ref())
            .expect("JSON should contain Pre-Authorized Code grant");

        assert_eq!(grant_pre_auth.pre_authorized_code.as_ref(), "oaKazRN8I0IbtZ0C7JuMn5");

        let Some(tx_code) = &grant_pre_auth.tx_code else {
            panic!("JSON should contain Pre-Authorized Code transaction code");
        };

        assert_eq!(tx_code.input_mode, Some(PreAuthTransactionCodeInputMode::Numeric));
        assert_eq!(tx_code.length, Some(4));
        assert_eq!(
            tx_code.description.as_deref(),
            Some("Please provide the one-time code that was sent via e-mail")
        );

        assert_eq!(serde_json::to_value(credential_offer).unwrap(), credential_offer_json);
    }

    #[test]
    fn test_credential_offer_container_by_value_deserialization() {
        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.2-3
        let offer_uri = "openid-credential-offer://?credential_offer=%7B%22credential_issuer%22:%22https:\
                         //credential-issuer.example.com%22,%22credential_configuration_ids%22:%5B%22org.\
                         iso.18013.5.1.mDL%22%5D,%22grants%22:%7B%22urn:ietf:params:oauth:grant-type:pre-\
                         authorized_code%22:%7B%22pre-authorized_code%22:%22oaKazRN8I0IbtZ0C7JuMn5%22,%22\
                         tx_code%22:%7B%22input_mode%22:%22text%22,%22description%22:%22Please%20enter%20\
                         the%20serial%20number%20of%20your%20physical%20drivers%20license%22%7D%7D%7D%7D";

        let uri = offer_uri.parse::<Url>().unwrap();
        let offer_container = serde_qs::from_str::<CredentialOfferContainer>(uri.query().unwrap())
            .expect("should be able to deserialize CredentialOfferContainer");

        let CredentialOfferContainer::CredentialOffer(credential_offer) = offer_container else {
            panic!("URI should contain Credential Offer by value");
        };

        assert_eq!(
            credential_offer.credential_issuer.as_ref(),
            "https://credential-issuer.example.com"
        );
        assert!(
            credential_offer
                .credential_configuration_ids
                .iter()
                .map(AsRef::as_ref)
                .eq(["org.iso.18013.5.1.mDL"])
        );

        let grant_pre_auth = credential_offer
            .grants
            .and_then(|grants| grants.pre_authorized_code)
            .expect("URI should contain Pre-Authorized Code grant");

        assert_eq!(grant_pre_auth.pre_authorized_code.as_ref(), "oaKazRN8I0IbtZ0C7JuMn5");

        let Some(tx_code) = grant_pre_auth.tx_code else {
            panic!("URI should contain Pre-Authorized Code transaction code");
        };

        assert_eq!(tx_code.input_mode, Some(PreAuthTransactionCodeInputMode::Text));
        assert!(tx_code.length.is_none());
        assert_eq!(
            tx_code.description.as_deref(),
            Some("Please enter the serial number of your physical drivers license")
        );
    }

    #[test]
    fn test_credential_offer_container_by_reference_deserialization() {
        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-4.1.3-7
        let offer_uri = "openid-credential-offer://?credential_offer_uri=https%3A%2F%2Fserver%2Eexample%2Ecom%\
                         2Fcredential-offer%2FGkurKxf5T0Y-mnPFCHqWOMiZi4VS138cQO_V7PZHAdM";

        let uri = offer_uri.parse::<Url>().unwrap();
        let offer_container = serde_qs::from_str::<CredentialOfferContainer>(uri.query().unwrap())
            .expect("should be able to deserialize CredentialOfferContainer");

        let CredentialOfferContainer::CredentialOfferUri(credential_offer_uri) = offer_container else {
            panic!("URI should contain Credential Offer by reference");
        };

        assert_eq!(
            credential_offer_uri.as_url().as_str(),
            "https://server.example.com/credential-offer/GkurKxf5T0Y-mnPFCHqWOMiZi4VS138cQO_V7PZHAdM"
        );
    }

    #[test]
    fn test_credential_offer_container_into_query_pair_offer() {
        let offer = CredentialOffer::new_pre_authorized(
            "https://issuer.example.com".parse().unwrap(),
            vec!["MyCredential".to_string().into()].try_into().unwrap(),
            "abc123".to_string().into(),
        );

        let container = CredentialOfferContainer::new_offer(offer);
        let (key, value) = container.into_query_pair();

        assert_eq!(key, "credential_offer");
        assert_eq!(
            value,
            "{\
                \"credential_issuer\":\"https://issuer.example.com\",\
                \"credential_configuration_ids\":[\"MyCredential\"],\
                    \"grants\":{\
                    \"urn:ietf:params:oauth:grant-type:pre-authorized_code\":{\
                        \"pre-authorized_code\":\"abc123\"\
            }}}"
        );
        let parsed: CredentialOffer = serde_json::from_str(&value).expect("value should be valid CredentialOffer JSON");
        assert_eq!(parsed.credential_issuer.as_ref(), "https://issuer.example.com");
    }

    #[test]
    fn test_credential_offer_container_into_query_pair_uri() {
        let uri: IssuerUrl = "https://issuer.example.com/offer/123".parse().unwrap();
        let container = CredentialOfferContainer::new_uri(uri);

        let (key, value) = container.into_query_pair();

        assert_eq!(key, "credential_offer_uri");
        assert_eq!(value, "https://issuer.example.com/offer/123");
    }

    #[test]
    fn test_credential_offer_container_to_credential_offer_url() {
        let offer = CredentialOffer::new_pre_authorized(
            "https://issuer.example.com".parse().unwrap(),
            vec!["MyCredential".to_string().into()].try_into().unwrap(),
            "abc123".to_string().into(),
        );
        let container = CredentialOfferContainer::new_offer(offer);

        let url = container.to_credential_offer_url();

        assert_eq!(url.scheme(), OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME);
        assert!(url.query().is_some_and(|q| q.contains("credential_offer")));
        assert_eq!(
            url.as_str(),
            "openid-credential-offer://?credential_offer=%7B%22credential_issuer%22%3A%22https%3A%2F%2Fissuer.example.\
             com%22%2C%22credential_configuration_ids%22%3A%5B%22MyCredential%22%5D%2C%22grants%22%3A%7B%22urn%3Aietf%\
             3Aparams%3Aoauth%3Agrant-type%3Apre-authorized_code%22%3A%7B%22pre-authorized_code%22%3A%22abc123%22%7D%\
             7D%7D"
        );
    }

    #[test]
    fn test_credential_offer_container_uri_to_credential_offer_url() {
        let uri: IssuerUrl = "https://issuer.example.com/offer/123".parse().unwrap();
        let container = CredentialOfferContainer::new_uri(uri);

        let url = container.to_credential_offer_url();

        assert_eq!(url.scheme(), OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME);
        assert!(url.query().is_some_and(|q| q.contains("credential_offer_uri")));
        assert_eq!(
            url.as_str(),
            "openid-credential-offer://?credential_offer_uri=https%3A%2F%2Fissuer.example.com%2Foffer%2F123"
        );
    }
}
