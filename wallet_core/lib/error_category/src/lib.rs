#[cfg(feature = "sentry")]
pub mod sentry;

pub use error_category_derive::ErrorCategory;
pub use error_category_derive::sentry_capture_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Category {
    Expected, // Don't report to Sentry
    Critical, // Report Error to Sentry with contents
    #[strum(serialize = "pd")]
    PersonalData, // Report Error to Sentry without contents
    Unexpected, // Should never occur in the Wallet, log error and report to Sentry without contents
}

pub trait ErrorCategory {
    fn category(&self) -> Category;
}
