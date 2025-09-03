use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use attestation_types::claim_path::ClaimPath;
use crypto::x509::CertificateError;
use dcql::CredentialFormat;
use dcql::disclosure::DisclosedCredential;
use http_utils::urls::HttpsUri;
use mdoc::DataElementIdentifier;
use mdoc::DataElementValue;
use mdoc::NameSpace;
use mdoc::holder::disclosure::claim_path_to_mdoc_path;
use mdoc::verifier::DisclosedDocument;
use sd_jwt::sd_jwt::SdJwt;
use utils::vec_at_least::VecNonEmpty;

use crate::attributes::AttributeError;
use crate::attributes::AttributeValue;
use crate::attributes::Attributes;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidityInfo {
    pub signed: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

impl TryFrom<&mdoc::iso::ValidityInfo> for ValidityInfo {
    type Error = chrono::ParseError;

    fn try_from(value: &mdoc::iso::ValidityInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            signed: (&value.signed).try_into()?,
            valid_from: Some((&value.valid_from).try_into()?),
            valid_until: Some((&value.valid_until).try_into()?),
        })
    }
}

#[cfg(feature = "test")]
impl From<ValidityInfo> for mdoc::iso::ValidityInfo {
    fn from(value: ValidityInfo) -> Self {
        Self {
            signed: value.signed.into(),
            valid_from: value.valid_from.unwrap().into(),
            valid_until: value.valid_until.unwrap().into(),
            expected_update: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "test", derive(derive_more::Unwrap))]
#[serde(tag = "format", content = "attributes", rename_all = "snake_case")]
pub enum DisclosedAttributes {
    MsoMdoc(IndexMap<NameSpace, IndexMap<String, AttributeValue>>),
    #[serde(rename = "dc+sd-jwt")]
    SdJwt(Attributes),
}

impl TryFrom<IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>> for DisclosedAttributes {
    type Error = AttributeError;

    fn try_from(
        map: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
    ) -> Result<Self, Self::Error> {
        Ok(DisclosedAttributes::MsoMdoc(
            map.into_iter()
                .map(|(namespace, attributes)| {
                    Ok((
                        namespace,
                        attributes
                            .into_iter()
                            .map(|(key, value)| Ok((key, value.try_into()?)))
                            .collect::<Result<_, Self::Error>>()?,
                    ))
                })
                .collect::<Result<_, Self::Error>>()?,
        ))
    }
}

impl TryFrom<serde_json::Map<String, serde_json::Value>> for DisclosedAttributes {
    type Error = AttributeError;

    fn try_from(map: serde_json::Map<String, serde_json::Value>) -> Result<Self, Self::Error> {
        Ok(DisclosedAttributes::SdJwt(
            map.into_iter()
                .map(|(key, attributes)| Ok((key, attributes.try_into()?)))
                .collect::<Result<IndexMap<_, _>, Self::Error>>()?
                .into(),
        ))
    }
}

#[cfg(feature = "test")]
impl From<DisclosedAttributes> for IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>> {
    fn from(attributes: DisclosedAttributes) -> Self {
        attributes
            .unwrap_mso_mdoc()
            .into_iter()
            .map(|(namespace, attributes)| {
                (
                    namespace,
                    attributes.into_iter().map(|(key, value)| (key, value.into())).collect(),
                )
            })
            .collect()
    }
}

/// Attestation that was disclosed; consisting of attributes, validity information, issuer URI and the issuer CA's
/// common name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisclosedAttestation {
    pub attestation_type: String,
    #[serde(flatten)]
    pub attributes: DisclosedAttributes,
    pub issuer_uri: HttpsUri,

    /// The issuer CA's common name
    pub ca: String,
    pub validity_info: ValidityInfo,
}

impl TryFrom<DisclosedDocument> for DisclosedAttestation {
    type Error = DisclosedAttestationError;

    fn try_from(doc: DisclosedDocument) -> Result<Self, Self::Error> {
        Ok(DisclosedAttestation {
            attestation_type: doc.doc_type,
            attributes: doc.attributes.try_into()?,
            issuer_uri: doc.issuer_uri,
            ca: doc.ca,
            validity_info: (&doc.validity_info).try_into()?,
        })
    }
}

impl TryFrom<SdJwt> for DisclosedAttestation {
    type Error = DisclosedAttestationError;

    fn try_from(sd_jwt: SdJwt) -> Result<Self, Self::Error> {
        let claims = sd_jwt.claims();
        let validity_info = ValidityInfo {
            signed: claims.iat.clone().into_inner().into(),
            valid_from: claims.nbf.map(Into::into),
            valid_until: claims.exp.map(Into::into),
        };

        let ca = sd_jwt
            .issuer_certificate()
            .ok_or(DisclosedAttestationError::MissingIssuerCertificate)?
            .issuer_common_names()?
            .first()
            .ok_or(DisclosedAttestationError::EmptyIssuerCommonName)?
            .to_string();
        let attributes = sd_jwt.to_disclosed_object()?.try_into()?;
        Ok(DisclosedAttestation {
            attestation_type: claims
                .vct
                .as_ref()
                .ok_or(DisclosedAttestationError::MissingAttributes("vct"))?
                .to_owned(),
            attributes,
            issuer_uri: claims.iss.clone().into_inner(),
            ca,
            validity_info,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DisclosedAttestationError {
    #[error("error converting mdoc attributes: {0}")]
    AttributeError(#[from] AttributeError),

    #[error("parse error while converting validity_info: {0}")]
    ParseError(#[from] chrono::ParseError),

    #[error("missing SD JWT claim: {0}")]
    MissingAttributes(&'static str),

    #[error("error converting SD JWT to disclosed object: {0}")]
    DisclosedObjectConversion(#[from] sd_jwt::error::Error),

    #[error("missing issuer certificate in SD JWT")]
    MissingIssuerCertificate,

    #[error("issuer common name in SD JWT issuer certificate is not a string")]
    IssuerCommonNameNotAString(#[from] CertificateError),

    #[error("empty issuer common name in SD JWT issuer certificate")]
    EmptyIssuerCommonName,
}

impl DisclosedCredential for DisclosedAttestation {
    fn format(&self) -> CredentialFormat {
        match &self.attributes {
            DisclosedAttributes::MsoMdoc(_) => CredentialFormat::MsoMdoc,
            DisclosedAttributes::SdJwt(_) => CredentialFormat::SdJwt,
        }
    }

    fn credential_type(&self) -> &str {
        &self.attestation_type
    }

    fn missing_claim_paths<'a, 'b>(
        &'a self,
        request_claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> HashSet<VecNonEmpty<ClaimPath>> {
        request_claim_paths
            .into_iter()
            .flat_map(|claim_path| {
                let attribute_present = match &self.attributes {
                    DisclosedAttributes::MsoMdoc(name_spaces) => claim_path_to_mdoc_path(claim_path)
                        .and_then(|(name_space_id, attribute_id)| {
                            name_spaces
                                .get(name_space_id)
                                .map(|name_space| name_space.contains_key(attribute_id))
                        })
                        .unwrap_or(false),
                    DisclosedAttributes::SdJwt(attributes) => attributes.has_claim_path(claim_path),
                };

                (!attribute_present).then(|| claim_path.clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use chrono::Utc;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;

    use attestation_types::claim_path::ClaimPath;
    use dcql::CredentialFormat;
    use dcql::disclosure::DisclosedCredential;
    use mdoc::examples::EXAMPLE_ATTRIBUTES;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use utils::vec_at_least::NonEmptyIterator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;

    use super::DisclosedAttestation;
    use super::DisclosedAttributes;
    use super::ValidityInfo;

    impl DisclosedAttestation {
        fn mdoc_example() -> Self {
            Self {
                attestation_type: EXAMPLE_DOC_TYPE.to_string(),
                attributes: DisclosedAttributes::MsoMdoc(IndexMap::from([(
                    EXAMPLE_NAMESPACE.to_string(),
                    EXAMPLE_ATTRIBUTES
                        .iter()
                        .map(|attribute| (attribute.to_string(), AttributeValue::Null))
                        .collect(),
                )])),
                issuer_uri: "https://example.com".parse().unwrap(),
                ca: "Example CA".to_string(),
                validity_info: ValidityInfo {
                    signed: Utc::now(),
                    valid_from: None,
                    valid_until: None,
                },
            }
        }

        fn sd_jwt_example() -> Self {
            Self {
                attestation_type: EXAMPLE_DOC_TYPE.to_string(),
                attributes: DisclosedAttributes::SdJwt(
                    IndexMap::from([
                        ("family_name".to_string(), Attribute::Single(AttributeValue::Null)),
                        (
                            "address".to_string(),
                            Attribute::Nested(IndexMap::from([(
                                "street".to_string(),
                                Attribute::Single(AttributeValue::Null),
                            )])),
                        ),
                    ])
                    .into(),
                ),
                issuer_uri: "https://example.com".parse().unwrap(),
                ca: "Example CA".to_string(),
                validity_info: ValidityInfo {
                    signed: Utc::now(),
                    valid_from: None,
                    valid_until: None,
                },
            }
        }
    }

    #[rstest]
    #[case(json!([
        {
            "attestationType": "com.example.pid",
            "issuerUri": "https://pid.example.com",
            "ca": "ca.example.com",
            "validityInfo": {
                "signed": "2014-11-28 12:00:09 UTC",
                "validFrom": "2014-11-28 12:00:09 UTC",
                "validUntil": "2014-11-28 12:00:09 UTC"
            },
            "format": "mso_mdoc",
            "attributes": {
                "com.example.pid": {
                    "bsn": "0912345678"
                }
            }
        },
        {
        "attestationType": "com.example.address",
            "issuerUri": "https://pid.example.com",
            "ca": "ca.example.com",
            "validityInfo": {
                "signed": "2014-11-28 12:00:09 UTC",
                "validFrom": "2014-11-28 12:00:09 UTC",
                "validUntil": "2014-11-28 12:00:09 UTC"
            },
            "format": "mso_mdoc",
            "attributes": {
                "com.example.address": {
                    "street": "Hoofdstraat"
                }
            }
        }
    ]))]
    #[case(json!([
        {
            "attestationType": "com.example.pid",
            "issuerUri": "https://pid.example.com",
            "ca": "ca.example.com",
            "validityInfo": {
                "signed": "2014-11-28 12:00:09 UTC",
                "validFrom": "2014-11-28 12:00:09 UTC",
                "validUntil": "2014-11-28 12:00:09 UTC"
            },
            "format": "mso_mdoc",
            "attributes": {
                "com.example.pid": {
                    "bsn": "0912345678"
                }
            }
        },
        {
            "attestationType": "com.example.address",
            "issuerUri": "https://pid.example.com",
            "ca": "ca.example.com",
            "validityInfo": {
                "signed": "2014-11-28 12:00:09 UTC",
                "validFrom": "2014-11-28 12:00:09 UTC",
                "validUntil": "2014-11-28 12:00:09 UTC"
            },
            "format": "dc+sd-jwt",
            "attributes": {
                "address": {
                    "street": "Main St",
                    "house_number": 123,
                    "locality": "Anytown",
                    "region": "Anystate",
                    "country": "US"
                }
            }
        }
    ]))]
    #[case(json!([
        {
            "attestationType": "com.example.pid",
            "issuerUri": "https://pid.example.com",
            "ca": "ca.example.com",
            "validityInfo": {
                "signed": "2014-11-28 12:00:09 UTC",
                "validFrom": "2014-11-28 12:00:09 UTC",
                "validUntil": "2014-11-28 12:00:09 UTC"
            },
            "format": "dc+sd-jwt",
            "attributes": {
                "nationalities": [
                    "DE",
                    "NL"
                ],
                "is_over_65": true,
                "address": {
                    "street": "Main St",
                    "house": {
                        "number": 123,
                        "letter": "A"
                    }
                }
            }
        }
    ]))]
    fn serialize_disclosed_attestation_ok(#[case] attestations: serde_json::Value) {
        serde_json::from_value::<Vec<DisclosedAttestation>>(attestations).unwrap();
    }

    fn claim_path(elements: &VecNonEmpty<&str>) -> VecNonEmpty<ClaimPath> {
        elements
            .nonempty_iter()
            .map(|key| ClaimPath::SelectByKey(key.to_string()))
            .collect()
    }

    #[rstest]
    #[case(CredentialFormat::MsoMdoc, vec![], vec![])]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "family_name"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "issue_date"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "expiry_date"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "document_number"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "portrait"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "driving_privileges"]),
        ],
        vec![],
    )]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![claim_path(&vec_nonempty!["org.iso.18013.5.1", "is_rich"])],
        vec![claim_path(&vec_nonempty!["org.iso.18013.5.1", "is_rich"])],
    )]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "family_name"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "is_rich"]),
        ],
        vec![claim_path(&vec_nonempty!["org.iso.18013.5.1", "is_rich"])],
    )]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "family_name"]),
            claim_path(&vec_nonempty!["vroom", "driving_privileges"]),
        ],
        vec![claim_path(&vec_nonempty!["vroom", "driving_privileges"])],
    )]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "portrait"]),
            claim_path(&vec_nonempty!["foobar"]),
            claim_path(&vec_nonempty!["org.iso.18013.5.1", "driving_privileges"]),
            claim_path(&vec_nonempty!["foobar", "bleh", "blah"]),
        ],
        vec![
            claim_path(&vec_nonempty!["foobar", "bleh", "blah"]),
            claim_path(&vec_nonempty!["foobar"]),
        ],
    )]
    #[case(
        CredentialFormat::MsoMdoc,
        vec![vec_nonempty![ClaimPath::SelectAll]],
        vec![vec_nonempty![ClaimPath::SelectAll]]
    )]
    #[case(CredentialFormat::SdJwt, vec![], vec![])]
    #[case(
        CredentialFormat::SdJwt,
        vec![
            claim_path(&vec_nonempty!["address", "street"]),
            claim_path(&vec_nonempty!["family_name"]),
        ],
        vec![],
    )]
    #[case(
        CredentialFormat::SdJwt,
        vec![
            claim_path(&vec_nonempty!["family_name", "something"]),
            claim_path(&vec_nonempty!["address"]),
            claim_path(&vec_nonempty!["address", "house_number", "something"]),
        ],
        vec![
            claim_path(&vec_nonempty!["family_name", "something"]),
            claim_path(&vec_nonempty!["address"]),
            claim_path(&vec_nonempty!["address", "house_number", "something"]),
        ],
    )]
    #[case(
        CredentialFormat::SdJwt,
        vec![
            claim_path(&vec_nonempty!["address", "street"]),
            claim_path(&vec_nonempty!["address", "house_number"]),
        ],
        vec![claim_path(&vec_nonempty!["address", "house_number"])],
    )]
    fn test_disclosed_attestation_missing_claim_paths(
        #[case] credential_format: CredentialFormat,
        #[case] request_claim_paths: Vec<VecNonEmpty<ClaimPath>>,
        #[case] expected_missing_attributes: Vec<VecNonEmpty<ClaimPath>>,
    ) {
        let disclosed_attestation = match credential_format {
            CredentialFormat::MsoMdoc => DisclosedAttestation::mdoc_example(),
            CredentialFormat::SdJwt => DisclosedAttestation::sd_jwt_example(),
        };

        let missing_attributes = disclosed_attestation.missing_claim_paths(&request_claim_paths);

        assert_eq!(missing_attributes, expected_missing_attributes.into_iter().collect());
    }
}
