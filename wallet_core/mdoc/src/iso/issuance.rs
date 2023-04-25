use ciborium::value::Value;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ServiceEngagement {
    id: String,
    url: Option<ServerUrl>,
    pc: Option<ProvisioningCode>,
    #[serde(rename = "Opt")]
    opt: Option<Options>,
}

pub type ProvisioningCode = String;
pub type ServerUrl = String;
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

pub type SessionId = ByteBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "StartProvisioning")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct StartProvisioningMessage {
    provisioning_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "ReadyToProvision")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct ReadyToProvisionMessage {
    e_session_id: SessionId,
}

// Session termination

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "RequestEndSession")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestEndSessionMessage {
    e_session_id: SessionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "foo.bar.EndSession")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct EndSessionMessage {
    e_session_id: SessionId,
    reason: String, // "known values include success, failed, restart"
    delay: Option<u64>,
    #[serde(rename = "SED")]
    sed: String, // "e.g. new SED to be used by mdoc app to resume session"
}

#[cfg(test)]
mod tests {
    use crate::serialization::cbor_serialize;

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
        let deserialized: Options = ciborium::de::from_reader(serialized.as_slice()).unwrap();
        assert_eq!(map, deserialized);
    }
}
