mod attribute;
mod disclosure;
mod issuance;

use std::collections::HashSet;

use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use mdoc::utils::auth::Organization;
use openid4vc::attributes::AttributeError;
use openid4vc::attributes::AttributeValue;
use sd_jwt_vc_metadata::ClaimDisplayMetadata;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::JsonSchemaProperty;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum AttestationError {
    #[error("some attributes not processed by claim: {0:?}")]
    #[category(pd)]
    AttributeNotProcessedByClaim(HashSet<Vec<String>>),

    #[error("unable to convert into attestation attribute value: {0:?} having metadata: {1:?}")]
    #[category(pd)]
    AttributeConversion(AttributeValue, JsonSchemaProperty),

    #[error("unable to parse attribute value into date: {0:?}")]
    #[category(pd)]
    AttributeDateValue(#[from] chrono::ParseError),

    #[error("error converting from mdoc attribute: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub identity: AttestationIdentity,
    pub attestation_type: String,
    pub display_metadata: VecNonEmpty<DisplayMetadata>,
    pub issuer: Organization,
    pub attributes: Vec<AttestationAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttestationIdentity {
    Ephemeral,
    Fixed { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationAttribute {
    pub key: Vec<String>,
    pub metadata: Vec<ClaimDisplayMetadata>,
    pub value: AttestationAttributeValue,
    pub svg_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationAttributeValue {
    Basic(AttributeValue),
    Date(NaiveDate),
}
