#[cfg(feature = "sentry")]
pub mod sentry;

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

pub use error_category_derive::{sentry_capture_error, ErrorCategory};

const CRITICAL: &str = "critical";
const EXPECTED: &str = "expected";
const UNEXPECTED: &str = "unexpected";
const PD: &str = "pd";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Expected,     // Don't report to Sentry
    Critical,     // Report Error to Sentry with contents
    PersonalData, // Report Error to Sentry without contents
    Unexpected,   // Should never occur in the Wallet, log error and report to Sentry without contents
}

#[derive(Debug, thiserror::Error)]
#[error("not a category: {0}")]
pub struct UnrecognizedCategory(String);

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected => f.write_str(EXPECTED),
            Self::Critical => f.write_str(CRITICAL),
            Self::PersonalData => f.write_str(PD),
            Self::Unexpected => f.write_str(UNEXPECTED),
        }
    }
}

impl FromStr for Category {
    type Err = UnrecognizedCategory;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            EXPECTED => Ok(Self::Expected),
            CRITICAL => Ok(Self::Critical),
            PD => Ok(Self::PersonalData),
            UNEXPECTED => Ok(Self::Unexpected),
            _ => Err(UnrecognizedCategory(s.to_string())),
        }
    }
}

pub trait ErrorCategory {
    fn category(&self) -> Category;
}
