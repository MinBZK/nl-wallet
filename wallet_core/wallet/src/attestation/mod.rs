mod attribute;
mod disclosure;
mod issuance;

use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use nl_wallet_mdoc::utils::auth::Organization;
use openid4vc::attributes::AttributeError;
use openid4vc::attributes::AttributeValue;
use sd_jwt::metadata::ClaimDisplayMetadata;
use sd_jwt::metadata::ClaimPath;
use sd_jwt::metadata::DisplayMetadata;
use wallet_common::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum AttestationError {
    #[error("error selecting attribute for claim: {0:?}")]
    #[category(pd)]
    AttributeNotFoundForClaim(VecNonEmpty<ClaimPath>),

    #[error("some attributes not processed by claim: {0:?}")]
    #[category(pd)]
    AttributeNotProcessedByClaim(HashSet<Vec<String>>),

    #[error("error converting from mdoc attribute: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),
}

#[derive(Debug, Clone, Copy)]
enum AttributeSelectionMode {
    Issuance,
    Disclosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub identity: AttestationIdentity,
    pub attestation_type: String,
    pub display_metadata: HashMap<String, DisplayMetadata>,
    pub issuer: Organization,
    pub attributes: Vec<AttestationAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationIdentity {
    Ephemeral,
    Fixed { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationAttribute {
    pub key: Vec<String>,
    pub metadata: HashMap<String, ClaimDisplayMetadata>,
    pub value: AttributeValue,
}
