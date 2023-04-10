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
pub type Options = IndexMap<String, Value>; // TODO should allow only maps

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
