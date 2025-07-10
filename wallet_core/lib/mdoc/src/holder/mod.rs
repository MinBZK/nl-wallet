//! Holder software to store and disclose mdocs.

use error_category::ErrorCategory;

pub mod disclosure;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum HolderError {
    #[error("mdoc is missing type metadata integrity digest")]
    #[category(critical)]
    MissingMetadataIntegrity,
}
