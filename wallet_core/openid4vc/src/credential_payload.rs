use chrono::DateTime;
use chrono::ParseError;
use chrono::Utc;
use indexmap::IndexMap;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::TimestampSeconds;

use error_category::ErrorCategory;
use mdoc::holder::Mdoc;
use mdoc::unsigned::Entry;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::AttestationQualification;
use mdoc::NameSpace;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataError;
use wallet_common::urls::HttpsUri;

use crate::attributes::Attribute;
use crate::attributes::AttributeError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadError {
    #[error("unable to strip namespace '{namespace}' from key '{key}'")]
    #[category(pd)]
    NamespaceStripping { namespace: String, key: String },

    #[error("error converting to JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[from] serde_json::Error),

    #[error("metadata validation error: {0}")]
    #[category(pd)]
    Metadata(#[from] TypeMetadataError),

    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[from] ParseError),

    #[error("mdoc is missing issuer URI")]
    #[category(critical)]
    MissingIssuerUri,

    #[error("mdoc is missing attestation qualification")]
    #[category(critical)]
    MissingAttestationQualification,

    #[error("attribute error: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),
}

/// This struct represents the Claims Set received from the issuer. Its JSON representation should be verifiable by the
/// JSON schema defined in the SD-JWT VC Type Metadata (`TypeMetadata`).
///
/// Converting both an (unsigned) mdoc and SD-JWT document to this struct should yield the same result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct CredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "iss")]
    pub issuer: HttpsUri,

    #[serde(rename = "iat")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub issued_at: Option<DateTime<Utc>>,

    #[serde(rename = "exp")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub expires: Option<DateTime<Utc>>,

    #[serde(rename = "nbf")]
    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub not_before: Option<DateTime<Utc>>,

    pub attestation_qualification: AttestationQualification,

    #[serde(flatten)]
    pub attributes: IndexMap<String, Attribute>,
}

impl CredentialPayload {
    pub fn from_unsigned_mdoc(
        unsigned_mdoc: UnsignedMdoc,
        metadata: &TypeMetadata,
    ) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            metadata,
            unsigned_mdoc.attributes.into(),
            unsigned_mdoc.issuer_uri,
            Some(Utc::now()),
            Some((&unsigned_mdoc.valid_until).try_into()?),
            Some((&unsigned_mdoc.valid_from).try_into()?),
            unsigned_mdoc.attestation_qualification,
        )
    }

    pub fn from_mdoc(mdoc: Mdoc, metadata: &TypeMetadata) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_attributes(
            metadata,
            mdoc.issuer_signed.into_entries_by_namespace(),
            mdoc.mso.issuer_uri.ok_or(CredentialPayloadError::MissingIssuerUri)?,
            Some((&mdoc.mso.validity_info.signed).try_into()?),
            Some((&mdoc.mso.validity_info.valid_until).try_into()?),
            Some((&mdoc.mso.validity_info.valid_from).try_into()?),
            mdoc.mso
                .attestation_qualification
                .ok_or(CredentialPayloadError::MissingAttestationQualification)?,
        )
    }

    fn from_mdoc_attributes(
        metadata: &TypeMetadata,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
        issuer: HttpsUri,
        issued_at: Option<DateTime<Utc>>,
        expires: Option<DateTime<Utc>>,
        not_before: Option<DateTime<Utc>>,
        attestation_qualification: AttestationQualification,
    ) -> Result<Self, CredentialPayloadError> {
        let attributes = Attribute::from_mdoc_attributes(metadata, mdoc_attributes)?;

        let payload = Self {
            attestation_type: metadata.as_ref().vct.clone(),
            issuer,
            issued_at,
            expires,
            not_before,
            attestation_qualification,
            attributes,
        };

        payload.validate(metadata)?;

        Ok(payload)
    }

    fn validate(&self, metadata: &TypeMetadata) -> Result<(), CredentialPayloadError> {
        metadata.validate(&serde_json::to_value(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;
    use chrono::Utc;
    use indexmap::IndexMap;
    use serde_json::json;
    use serde_valid::json::ToJsonString;

    use sd_jwt_vc_metadata::TypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::credential_payload::CredentialPayload;

    #[test]
    fn test_serialize_deserialize_and_validate() {
        let payload = CredentialPayload {
            attestation_type: String::from("com.example.pid"),
            issuer: "https://com.example.org/pid/issuer".parse().unwrap(),
            issued_at: Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap()),
            expires: None,
            not_before: None,
            attestation_qualification: "QEAA".parse().unwrap(),
            attributes: IndexMap::from([
                (
                    String::from("birth_date"),
                    Attribute::Single(AttributeValue::Text(String::from("1963-08-12"))),
                ),
                (
                    String::from("place_of_birth"),
                    Attribute::Nested(IndexMap::from([
                        (
                            String::from("locality"),
                            Attribute::Single(AttributeValue::Text(String::from("The Hague"))),
                        ),
                        (
                            String::from("country"),
                            Attribute::Nested(IndexMap::from([
                                (
                                    String::from("name"),
                                    Attribute::Single(AttributeValue::Text(String::from("The Netherlands"))),
                                ),
                                (
                                    String::from("area_code"),
                                    Attribute::Single(AttributeValue::Integer(33)),
                                ),
                            ])),
                        ),
                    ])),
                ),
                (
                    String::from("financial"),
                    Attribute::Nested(IndexMap::from([
                        (String::from("has_debt"), Attribute::Single(AttributeValue::Bool(true))),
                        (String::from("has_job"), Attribute::Single(AttributeValue::Bool(false))),
                        (
                            String::from("debt_amount"),
                            Attribute::Single(AttributeValue::Integer(-10_000)),
                        ),
                    ])),
                ),
            ]),
        };

        let expected_json = json!({
            "vct": "com.example.pid",
            "iss": "https://com.example.org/pid/issuer",
            "iat": 61,
            "attestation_qualification": "QEAA",
            "birth_date": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 33
                }
            },
            "financial": {
                "has_debt": true,
                "has_job": false,
                "debt_amount": -10000
            }
        });

        assert_eq!(
            serde_json::to_value(payload).unwrap().to_json_string_pretty().unwrap(),
            expected_json.to_json_string_pretty().unwrap()
        );

        let payload = serde_json::from_value::<CredentialPayload>(expected_json).unwrap();

        let metadata = TypeMetadata::example();

        payload.validate(&metadata).unwrap();
    }
}
