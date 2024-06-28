pub use wallet_common_macros::{handle_error_category, ErrorCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Expected,     // Don't report to Sentry
    Critical,     // Report Error to Sentry, with contents
    PersonalData, // Report Error to Sentry, without contents
}

pub trait ErrorCategory {
    fn category(&self) -> Category;
}
