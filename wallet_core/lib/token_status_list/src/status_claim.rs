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
            idx: 0,
            uri: "https://example.com/statuslists/1".parse().unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_status_claim() {
        let example = json!({
            "status_list": {
                "idx": 0,
                "uri": "https://example.com/statuslists/1"
            }
        });
        let StatusClaim::StatusList(claim) = serde_json::from_value(example).unwrap();
        assert_eq!(claim.idx, 0);
        assert_eq!(claim.uri, "https://example.com/statuslists/1".parse().unwrap());
    }
}
