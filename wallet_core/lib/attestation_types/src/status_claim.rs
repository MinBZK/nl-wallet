use serde::Deserialize;
use serde::Serialize;
use url::Url;

/// By including a "status" claim in a Referenced Token, the Issuer is referencing a mechanism to retrieve status
/// information about this Referenced Token. This crate defines one possible member of the "status" object,
/// called "status_list".
///
/// ```json
/// "status": {
///     "status_list": {
///         "idx": 0,
///         "uri": "https://example.com/statuslists/1"
///     }
/// }
/// ```
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-referenced-token>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusClaim {
    StatusList(StatusListClaim),
    IdentifierList(IdentifierListInfo),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusListClaim {
    /// A non-negative Integer that represents the index to check for status information in the Status List for the
    /// current Referenced Token.
    pub idx: u32,

    /// URI that identifies the Status List Token containing the status information for the Referenced Token.
    pub uri: Url,
}

#[cfg(feature = "mock")]
impl StatusClaim {
    pub fn new_mock() -> Self {
        StatusClaim::StatusList(StatusListClaim {
            idx: 1,
            uri: "https://example.com/statuslists/1".parse().unwrap(),
        })
    }
}

/// The `identifier_list` element is a CBOR structure with the following CDDL. The value of the Identifier field shall
/// be unique per MSO.
///
/// ```cddl
/// IdentifierListInfo = {
///    "id" : Identifier,
///    "uri": URI,
///    ? "certificate": Certificate
///    * tstr => RFU
/// }
///
/// Identifier = bstr
/// URI = tstr
/// Certificate = bstr
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentifierListInfo {
    pub id: Vec<u8>,
    pub uri: Url,
    pub certificate: Option<Vec<u8>>,
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::status_list(json!({
        "status_list": {
            "idx": 0,
            "uri": "https://example.com/statuslists/1"
        }
    }), StatusClaim::StatusList(StatusListClaim {
        idx: 0,
        uri: "https://example.com/statuslists/1".parse().unwrap(),
    }))]
    #[case::identifier_list(json!({
        "identifier_list": {
            "id": hex::decode("cccc").unwrap(),
            "uri": "https://example.com/identifierlists/1",
            // "certificate": h'aa...'
        }
    }), StatusClaim::IdentifierList(IdentifierListInfo {
        id: [0xcc, 0xcc].to_vec(),
        uri: "https://example.com/identifierlists/1".parse().unwrap(),
        certificate: None,
    }))]
    fn test_deserialize_status_claim(#[case] value: serde_json::Value, #[case] expected: StatusClaim) {
        let claim: StatusClaim = serde_json::from_value(value).unwrap();
        assert_eq!(claim, expected);
    }
}
