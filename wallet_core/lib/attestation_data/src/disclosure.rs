use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use crypto::x509::CertificateError;
use http_utils::urls::HttpsUri;
use mdoc::DataElementIdentifier;
use mdoc::DataElementValue;
use mdoc::NameSpace;

use crate::attributes::AttributeError;
use crate::attributes::AttributeValue;
use crate::attributes::Attributes;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidityInfo {
    pub signed: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

impl TryFrom<&mdoc::iso::ValidityInfo> for ValidityInfo {
    type Error = chrono::ParseError;

    fn try_from(value: &mdoc::iso::ValidityInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            signed: (&value.signed).try_into()?,
            valid_from: (&value.valid_from).try_into()?,
            valid_until: (&value.valid_until).try_into()?,
        })
    }
}

#[cfg(feature = "test")]
impl From<ValidityInfo> for mdoc::iso::ValidityInfo {
    fn from(value: ValidityInfo) -> Self {
        Self {
            signed: value.signed.into(),
            valid_from: value.valid_from.into(),
            valid_until: value.valid_until.into(),
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

impl TryFrom<mdoc::verifier::DisclosedDocument> for DisclosedAttestation {
    type Error = DisclosedAttestationError;

    fn try_from(doc: mdoc::verifier::DisclosedDocument) -> Result<Self, Self::Error> {
        Ok(DisclosedAttestation {
            attestation_type: doc.doc_type,
            attributes: doc.attributes.try_into()?,
            issuer_uri: doc.issuer_uri,
            ca: doc.ca,
            validity_info: (&doc.validity_info).try_into()?,
        })
    }
}

#[cfg(feature = "test")]
impl From<DisclosedAttestation> for mdoc::verifier::DisclosedDocument {
    fn from(doc: DisclosedAttestation) -> Self {
        mdoc::verifier::DisclosedDocument {
            doc_type: doc.attestation_type,
            attributes: doc.attributes.into(),
            issuer_uri: doc.issuer_uri,
            ca: doc.ca,
            validity_info: doc.validity_info.into(),
        }
    }
}

impl TryFrom<sd_jwt::sd_jwt::SdJwt> for DisclosedAttestation {
    type Error = DisclosedAttestationError;

    fn try_from(sd_jwt: sd_jwt::sd_jwt::SdJwt) -> Result<Self, Self::Error> {
        let claims = sd_jwt.claims();
        let validity_info = ValidityInfo {
            signed: claims
                .iat
                .ok_or(DisclosedAttestationError::MissingAttributes("iat"))?
                .into(),
            valid_from: claims
                .nbf
                .ok_or(DisclosedAttestationError::MissingAttributes("nbf"))?
                .into(),
            valid_until: claims
                .exp
                .ok_or(DisclosedAttestationError::MissingAttributes("exp"))?
                .into(),
        };

        let attestation_type = claims
            .vct
            .as_ref()
            .ok_or(DisclosedAttestationError::MissingAttributes("vct"))?
            .clone();
        let issuer_uri = claims
            .iss
            .as_ref()
            .ok_or(DisclosedAttestationError::MissingAttributes("iss"))?
            .clone();
        let ca = sd_jwt
            .issuer_certificate()
            .ok_or(DisclosedAttestationError::MissingIssuerCertificate)?
            .issuer_common_names()?
            .first()
            .ok_or(DisclosedAttestationError::EmptyIssuerCommonName)?
            .to_string();
        let attributes = sd_jwt.to_disclosed_object()?.try_into()?;
        Ok(DisclosedAttestation {
            attestation_type,
            attributes,
            issuer_uri,
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

#[cfg(test)]
mod test {
    use rstest::rstest;
    use serde_json::json;

    use super::DisclosedAttestation;

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
}
