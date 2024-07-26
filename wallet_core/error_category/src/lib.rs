#[cfg(feature = "sentry")]
pub mod sentry;

pub use error_category_derive::{sentry_capture_error, ErrorCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Expected,     // Don't report to Sentry
    Critical,     // Report Error to Sentry, with contents
    PersonalData, // Report Error to Sentry, without contents
    Unexpected,   // Should never occer at runtime, panic!
}

pub trait ErrorCategory {
    fn category(&self) -> Category;
}
