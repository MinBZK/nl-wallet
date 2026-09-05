use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::disclosure::DisclosedAttributes;
use attestation_data::validity::IssuanceValidity;
use attestation_types::credential_format::Format;
use attestation_types::qualification::AttestationQualification;
use dcql::CredentialQueryIdentifier;
use dcql::unique_id_vec::MayHaveUniqueId;
use http_utils::urls::HttpsUri;
use indexmap::IndexMap;
use serde::Deserialize;
use utils::vec_at_least::VecNonEmpty;

/// Attributes of an attestation that was disclosed, but without the DisclosedAttributes enum. This way, we can
/// deserialize both formats without having to deal with the enum variants in the code that uses this struct.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(from = "DisclosedAttestation")]
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

impl From<DisclosedAttestation> for DemoDisclosedAttestation {
    fn from(value: DisclosedAttestation) -> Self {
        // Due to `Attributes` being a tagged enum, `IndexMap<String, IndexMap<String, Attribute>>` no longer neatly
        // deserializes as `Attributes`. This conversion fixes that.
        let (format, attributes) = match value.attributes {
            DisclosedAttributes::SdJwt(attributes) => (Format::SdJwt, attributes),
            DisclosedAttributes::MsoMdoc(namespaces) => (
                Format::MsoMdoc,
                namespaces
                    .into_iter()
                    .map(|(k, v)| (k, Attribute::Object(v)))
                    .collect::<IndexMap<_, _>>()
                    .into(),
            ),
        };

        Self {
            attestation_type: value.attestation_type,
            attributes,
            format,
            issuer_uri: value.issuer_uri,
            attestation_qualification: value.attestation_qualification,
            ca: value.ca,
            issuance_validity: value.issuance_validity,
        }
    }
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
    use attestation_data::attributes::Attribute;
    use attestation_data::disclosure::DisclosedAttestation;
    use attestation_data::disclosure::DisclosedAttributes;
    use attestation_types::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use chrono::DateTime;
    use indexmap::IndexMap;
    use serde_json::json;
    use token_status_list::verification::verifier::RevocationStatus;

    use super::*;

    #[test]
    fn test_deserialize_disclosed_attestations() {
        let attestations = vec![
            DisclosedAttestation {
                attestation_type: PID_ATTESTATION_TYPE.to_string(),
                attributes: DisclosedAttributes::SdJwt(
                    IndexMap::from_iter([
                        ("bsn".to_string(), Attribute::Text("999991772".to_string())),
                        ("birthdate".to_string(), Attribute::Text("2000-03-24".to_string())),
                        ("given_name".to_string(), Attribute::Text("Frouke".to_string())),
                        ("family_name".to_string(), Attribute::Text("Jansen".to_string())),
                    ])
                    .into(),
                ),
                issuer_uri: "https://issuer.example.com/".parse().unwrap(),
                attestation_qualification: AttestationQualification::default(),
                ca: "ca.issuer.example.com".to_string(),
                issuance_validity: IssuanceValidity::new(
                    DateTime::UNIX_EPOCH,
                    Some(DateTime::UNIX_EPOCH),
                    Some(DateTime::UNIX_EPOCH),
                ),
                revocation_status: Some(RevocationStatus::Valid),
                aki: vec![],
            },
            DisclosedAttestation {
                attestation_type: ADDRESS_ATTESTATION_TYPE.to_string(),
                attributes: DisclosedAttributes::MsoMdoc(IndexMap::from_iter([(
                    format!("{}.address", ADDRESS_ATTESTATION_TYPE),
                    IndexMap::from_iter([
                        ("postal_code".to_string(), Attribute::Text("3528BG".to_string())),
                        ("house_number".to_string(), Attribute::Number(51.into())),
                        (
                            "street_address".to_string(),
                            Attribute::Text("Groenewoudsedijk".to_string()),
                        ),
                    ]),
                )])),
                issuer_uri: "https://issuer.example.com/".parse().unwrap(),
                attestation_qualification: AttestationQualification::default(),
                ca: "ca.issuer.example.com".to_string(),
                issuance_validity: IssuanceValidity::new(
                    DateTime::UNIX_EPOCH,
                    Some(DateTime::UNIX_EPOCH),
                    Some(DateTime::UNIX_EPOCH),
                ),
                revocation_status: Some(RevocationStatus::Valid),
                aki: vec![],
            },
        ];

        let attestations: Vec<DemoDisclosedAttestation> =
            serde_json::from_str(&serde_json::to_string(&attestations).unwrap()).unwrap();

        let expected = json!([{
            "attestation_type": "urn:example:pid:nl:1",
            "attributes": {
                "bsn": {
                    "type": "text",
                    "value": "999991772"
                },
                "birthdate": {
                    "type": "text",
                    "value": "2000-03-24"
                },
                "given_name": {
                    "type": "text",
                    "value": "Frouke"
                },
                "family_name": {
                    "type": "text",
                    "value": "Jansen"
                }
            },
            "format": "dc+sd-jwt",
            "issuer_uri": "https://issuer.example.com/",
            "attestation_qualification": "EAA",
            "ca": "ca.issuer.example.com",
            "issuance_validity": {
                "signed": "1970-01-01T00:00:00Z",
                "validFrom": "1970-01-01T00:00:00Z",
                "validUntil": "1970-01-01T00:00:00Z"
            }
        }, {
            "attestation_type": "urn:example:pid-address:nl:1",
            "attributes": {
                "urn:example:pid-address:nl:1.address": {
                    "type": "object",
                    "value": {
                        "postal_code": {
                            "type": "text",
                            "value": "3528BG"
                        },
                        "house_number": {
                            "type": "number",
                            "value": 51
                        },
                        "street_address": {
                            "type": "text",
                            "value": "Groenewoudsedijk"
                        }
                    }
                }
            },
            "format": "mso_mdoc",
            "issuer_uri": "https://issuer.example.com/",
            "attestation_qualification": "EAA",
            "ca": "ca.issuer.example.com",
            "issuance_validity": {
                "signed": "1970-01-01T00:00:00Z",
                "validFrom": "1970-01-01T00:00:00Z",
                "validUntil": "1970-01-01T00:00:00Z"
            }
        }]);

        assert_eq!(serde_json::to_value(&attestations).unwrap(), expected);
    }
}
