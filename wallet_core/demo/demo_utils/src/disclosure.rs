use serde::Deserialize;

use attestation_data::attributes::Attributes;
use attestation_data::disclosure::ValidityInfo;
use http_utils::urls::HttpsUri;
use openid4vc::Format;

/// Attributes of an attestation that was disclosed, but without the DisclosedAttributes enum. This way, we can
/// deserialize both formats without having to deal with the enum variants in the code that uses this struct.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoDisclosedAttestation {
    pub attestation_type: String,
    pub attributes: Attributes,
    pub format: Format,
    pub issuer_uri: HttpsUri,

    /// The issuer CA's common name
    pub ca: String,
    pub validity_info: ValidityInfo,
}

#[cfg(test)]
mod test {
    use indexmap::IndexMap;

    use serde_with::chrono::DateTime;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::disclosure::DisclosedAttestation;
    use attestation_data::disclosure::DisclosedAttributes;

    use super::*;

    #[test]
    fn test_deserialize_disclosed_attestations() {
        let attestations = vec![
            DisclosedAttestation {
                attestation_type: "urn:eudi:pid:nl:1".to_string(),
                attributes: DisclosedAttributes::MsoMdoc(IndexMap::from_iter(vec![(
                    "urn:eudi:pid:nl:1".to_string(),
                    IndexMap::from_iter(vec![
                        ("bsn".to_string(), AttributeValue::Text("999991772".to_string())),
                        ("birthdate".to_string(), AttributeValue::Text("2000-03-24".to_string())),
                        ("given_name".to_string(), AttributeValue::Text("Frouke".to_string())),
                        ("family_name".to_string(), AttributeValue::Text("Jansen".to_string())),
                    ]),
                )])),
                issuer_uri: "https://cert.issuer.example.com/".parse().unwrap(),
                ca: "ca.issuer.example.com".to_string(),
                validity_info: ValidityInfo {
                    signed: DateTime::UNIX_EPOCH,
                    valid_from: Some(DateTime::UNIX_EPOCH),
                    valid_until: Some(DateTime::UNIX_EPOCH),
                },
            },
            DisclosedAttestation {
                attestation_type: "urn:eudi:pid-address:nl:1".to_string(),
                attributes: DisclosedAttributes::SdJwt(
                    IndexMap::from_iter(vec![(
                        "address".to_string(),
                        Attribute::Nested(IndexMap::from_iter(vec![
                            (
                                "postal_code".to_string(),
                                Attribute::Single(AttributeValue::Text("3528BG".to_string())),
                            ),
                            (
                                "house_number".to_string(),
                                Attribute::Single(AttributeValue::Integer(51)),
                            ),
                            (
                                "street_address".to_string(),
                                Attribute::Single(AttributeValue::Text("Groenewoudsedijk".to_string())),
                            ),
                        ])),
                    )])
                    .into(),
                ),
                issuer_uri: "https://cert.issuer.example.com/".parse().unwrap(),
                ca: "ca.issuer.example.com".to_string(),
                validity_info: ValidityInfo {
                    signed: DateTime::UNIX_EPOCH,
                    valid_from: Some(DateTime::UNIX_EPOCH),
                    valid_until: Some(DateTime::UNIX_EPOCH),
                },
            },
        ];

        let attestations: Vec<DemoDisclosedAttestation> =
            serde_json::from_str(&serde_json::to_string(&attestations).unwrap()).unwrap();

        assert_eq!(attestations.len(), 2);
    }
}
