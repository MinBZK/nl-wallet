use serde::Deserialize;

use attestation_data::attributes::Attributes;
use attestation_data::validity::IssuanceValidity;
use attestation_types::qualification::AttestationQualification;
use dcql::CredentialQueryIdentifier;
use dcql::unique_id_vec::MayHaveUniqueId;
use http_utils::urls::HttpsUri;
use openid4vc::Format;
use utils::vec_at_least::VecNonEmpty;

/// Attributes of an attestation that was disclosed, but without the DisclosedAttributes enum. This way, we can
/// deserialize both formats without having to deal with the enum variants in the code that uses this struct.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DemoDisclosedAttestation {
    pub attestation_type: String,
    pub attributes: Attributes,
    pub format: Format,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,

    /// The issuer CA's common name
    pub ca: String,
    pub issuance_validity: IssuanceValidity,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DemoDisclosedAttestations {
    /// The identifier of the [`dcql::CredentialQuery`] that this attestation is a disclosure of.
    pub id: CredentialQueryIdentifier,

    pub attestations: VecNonEmpty<DemoDisclosedAttestation>,
}

impl MayHaveUniqueId for DemoDisclosedAttestations {
    fn id(&self) -> Option<&str> {
        Some(self.id.as_ref())
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use indexmap::IndexMap;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::disclosure::DisclosedAttestation;
    use attestation_data::disclosure::DisclosedAttributes;
    use token_status_list::verification::verifier::RevocationStatus;

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
                attestation_qualification: AttestationQualification::default(),
                ca: "ca.issuer.example.com".to_string(),
                issuance_validity: IssuanceValidity::new(
                    DateTime::UNIX_EPOCH,
                    Some(DateTime::UNIX_EPOCH),
                    Some(DateTime::UNIX_EPOCH),
                ),
                revocation_status: Some(RevocationStatus::Valid),
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
                attestation_qualification: AttestationQualification::default(),
                ca: "ca.issuer.example.com".to_string(),
                issuance_validity: IssuanceValidity::new(
                    DateTime::UNIX_EPOCH,
                    Some(DateTime::UNIX_EPOCH),
                    Some(DateTime::UNIX_EPOCH),
                ),
                revocation_status: Some(RevocationStatus::Valid),
            },
        ];

        let attestations: Vec<DemoDisclosedAttestation> =
            serde_json::from_str(&serde_json::to_string(&attestations).unwrap()).unwrap();

        assert_eq!(attestations.len(), 2);
    }
}
