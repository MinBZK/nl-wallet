use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use attestation_data::attributes::Attributes;
use attestation_data::disclosure::ValidityInfo;
use http_utils::urls::HttpsUri;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialFormat {
    Mdoc,
    SdJwt,
}

/// Attributes of an attestation that was disclosed, but without the DisclosedAttributes enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDisclosedAttributes {
    pub attributes: Attributes,
    pub format: CredentialFormat,
    pub issuer_uri: HttpsUri,

    /// The issuer CA's common name
    pub ca: String,
    pub validity_info: ValidityInfo,
}

pub type DisclosedAttestations = IndexMap<String, DocumentDisclosedAttributes>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_disclosed_attestations() {
        let json = r#"
        {
            "urn:eudi:pid:nl:1": {
                "format": "Mdoc",
                "attributes": {
                    "urn:eudi:pid:nl:1": {
                        "bsn": "999991772",
                        "birthdate": "2000-03-24",
                        "given_name": "Frouke",
                        "family_name": "Jansen"
                    }
                },
                "issuerUri": "https://cert.issuer.example.com/",
                "ca": "ca.issuer.example.com",
                "validityInfo": {
                    "signed": "2025-06-20T14:29:40Z",
                    "validFrom": "2025-06-20T00:00:00Z",
                    "validUntil": "2026-06-20T00:00:00Z"
                }
            },
            "urn:eudi:pid-address:nl:1": {
                "format": "Mdoc",
                "attributes": {
                    "urn:eudi:pid-address:nl:1.address": {
                        "postal_code": "3528BG",
                        "house_number": "51",
                        "street_address": "Groenewoudsedijk"
                    }
                },
                "issuerUri": "https://cert.issuer.example.com/",
                "ca": "ca.issuer.example.com",
                "validityInfo": {
                    "signed": "2025-06-20T14:29:40Z",
                    "validFrom": "2025-06-20T00:00:00Z",
                    "validUntil": "2026-06-20T00:00:00Z"
                }
            }
        }
        "#;

        let attestations: DisclosedAttestations = serde_json::from_str(json).unwrap();
        assert_eq!(attestations.len(), 2);
    }
}
