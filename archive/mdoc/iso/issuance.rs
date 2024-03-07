//! Issuance structures defined by ISO 23220-3. These are generic, i.e. not part of any application-specific
//! protocol such as BasicSA or the one defined in `basic_sa_ext.rs`.
//!
//! All protocol messages (except for the very first one, [`ServiceEngagement`]) of ISO 23220-3 contain a field
//! `messageType` which must contain a `tstr` constant associated to the protocol message type. In the messages below
//! this is implemented using the following attributes:
//!
//! ```rust
//! # #[derive(serde::Serialize, serde::Deserialize)]
//! #[serde(rename = "NameOfTheProtocolMessage")]
//! #[serde(tag = "messageType")]
//! pub struct SomeStruct {}
//! ```
//!
//! This results in `serde` including `"messageType": "NameOfTheProtocolMessage"` in the message during deserialization,
//! but not in enforcing that `messageType` has the expected value during deserialization. Instead, this is to be
//! enforced where applicable by the using code.

use std::{borrow::Cow, fmt::Display};

use ciborium::value::Value;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use url::Url;

use crate::utils::serialization::{RequiredValue, RequiredValueTrait};

/// First message of the issuer to be sent to the holder, e.g. in a QR code, scheme URL or universal link.
/// Contains the URL with which the holder can start the session, by sending a [`StartProvisioningMessage`] and
/// receiving a [`ReadyToProvisionMessage`].
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "UPPERCASE")]
pub struct ServiceEngagement {
    pub id: RequiredValue<ServiceEngagementID>,
    pub url: Option<ServerUrl>,
    pub pc: Option<ProvisioningCode>,
    #[serde(rename = "Opt")]
    pub opt: Option<Options>,
}

/// Hardcoded string part of every [`ServiceEngagement`].
#[derive(Debug, Clone)]
pub struct ServiceEngagementID;
impl RequiredValueTrait for ServiceEngagementID {
    type Type = Cow<'static, str>;
    const REQUIRED_VALUE: Self::Type = Cow::Borrowed("org.iso.23220-3-1.0");
}

/// For use in a [`ServiceEngagement`], "a unique identifier for the mobile document to be issued to the mdoc app.
/// It may be limited in time and scope and is typically created by the Issuing Authority".
pub type ProvisioningCode = String;

/// URL at which the holder can start a session, by sending a [`StartProvisioningMessage`] to it.
pub type ServerUrl = Url;

/// Free-form additional information.
/// "Optional fields defined by this standard will use non-negative integers as keys. Applications may use the options
/// field but only with keys of type tstr."
pub type Options = IndexMap<OptionsKey, Value>;

/// Key for options in the [`Options`] map.
/// Options defined by ISO 23220-3 use non-negative integers as keys. All other options must use tstr
/// in the format [Reverse Domain].[Domain Specific Extension].[Key Name].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum OptionsKey {
    Uint(u64),
    Tstr(String),
}

/// Identifies a session. Determined by the issuer when the session starts. Must be present in all issuance
/// protocol messages (except for the first message by the holder to the issuer, at which time the value is
/// not yet known.)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(ByteBuf);
impl From<ByteBuf> for SessionId {
    fn from(value: ByteBuf) -> Self {
        SessionId(value)
    }
}
impl From<Vec<u8>> for SessionId {
    fn from(value: Vec<u8>) -> Self {
        ByteBuf::from(value).into()
    }
}
impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
impl From<SessionId> for Vec<u8> {
    fn from(value: SessionId) -> Self {
        value.0.into_vec()
    }
}

pub const START_PROVISIONING_MSG_TYPE: &str = "StartProvisioning";

/// Holder -> issuer. Starts the session and indicates that we want to speak ISO 23220-3.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename = "StartProvisioning")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct StartProvisioningMessage {
    pub provisioning_code: Option<String>,
}

/// Issuer -> holder, reply to [`StartProvisioningMessage`]: agree with doing issuance. Includes the [`SessionId`] to
/// be used in all further protocol messages in the remainder of the session.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "ReadyToProvision")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct ReadyToProvisionMessage {
    pub e_session_id: SessionId,
}

// Session termination

pub const REQUEST_END_SESSION_MSG_TYPE: &str = "RequestEndSession";

/// Holder -> issuer. Can be sent at any time during the protocol to indicate to the issuer that the holder
/// wishes to abort.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "RequestEndSession")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestEndSessionMessage {
    pub e_session_id: SessionId,
}

/// Issuer -> holder. Indicates that the session is aborted. Can be sent as a reply to [`RequestEndSessionMessage`],
/// but also in other cases if the issuer itself decides to abort.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "EndSession")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct EndSessionMessage {
    pub e_session_id: SessionId,
    pub reason: String, // "known values include success, failed, restart"
    pub delay: Option<u64>,
    #[serde(rename = "SED")]
    pub sed: Option<String>, // "e.g. new SED to be used by mdoc app to resume session"
}

#[cfg(test)]
mod tests {
    use crate::utils::serialization::{cbor_deserialize, cbor_serialize};

    use super::*;

    #[test]
    fn test_options() {
        let map = Options::from([
            (OptionsKey::Tstr("hello".into()), Value::Text("world".into())),
            (OptionsKey::Uint(1), Value::Integer(42.into())),
        ]);

        // Explicitly assert CBOR structure of the serialized data
        assert_eq!(
            Value::serialized(&map).unwrap(),
            Value::Map(vec![
                (Value::Text("hello".into()), Value::Text("world".into())),
                (Value::Integer(1.into()), Value::Integer(42.into()))
            ])
        );

        // Check that we can deserialize to the same value
        let serialized = cbor_serialize(&map).unwrap();
        let deserialized: Options = cbor_deserialize(serialized.as_slice()).unwrap();
        assert_eq!(map, deserialized);
    }
}
