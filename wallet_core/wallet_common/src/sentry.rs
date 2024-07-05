use std::{borrow::Cow, error::Error};

use sentry::{event_from_error, ClientInitGuard, ClientOptions};
use serde::Deserialize;

use crate::error_category::{Category, ErrorCategory};

#[derive(Clone, Deserialize)]
pub struct Sentry {
    pub dsn: String,
    pub environment: String,
}

impl Sentry {
    /// Initialize Sentry and return the [`ClientInitGuard`].
    /// [release] must be set to `sentry::release_name!()` from the `main.rs` of the application, so that the release
    /// matches the main crate.
    #[must_use = "this value must be retained during the lifetime of the application"]
    pub fn init(&self, release: Option<Cow<'static, str>>) -> ClientInitGuard {
        sentry::init((
            self.dsn.clone(),
            ClientOptions {
                release,
                environment: Cow::from(self.environment.clone()).into(),
                debug: cfg!(debug_assertions),
                ..Default::default()
            },
        ))
    }
}

pub fn classify_and_report_to_sentry<T: ErrorCategory + Error>(error: T) -> T {
    match error.category() {
        Category::Expected => {}
        Category::Critical => {
            let _uuid = sentry::capture_error(&error);
        }
        Category::PersonalData => {
            let mut event = event_from_error(&error);
            // Clear all exception values, to remove privacy sensitive data
            event.exception.values.iter_mut().for_each(|value| value.value = None);
            let _uuid = sentry::capture_event(event);
        }
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;

    use sentry::{test::with_captured_events, Level};
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("Something wrong")]
    struct Error {
        category: Category,
    }

    impl ErrorCategory for Error {
        fn category(&self) -> Category {
            self.category
        }
    }

    #[test]
    fn test_classify_and_report_to_sentry_expected() {
        let error = Error {
            category: Category::Expected,
        };
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(error);
        });
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_classify_and_report_to_sentry_critical() {
        let error = Error {
            category: Category::Critical,
        };
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values[0].ty, "Error".to_string());
        assert_eq!(events[0].exception.values[0].value, Some("Something wrong".to_string()));
    }

    #[test]
    fn test_classify_and_report_to_sentry_personal_data() {
        let error = Error {
            category: Category::PersonalData,
        };
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values[0].ty, "Error".to_string());
        assert_eq!(events[0].exception.values[0].value, None);
    }
}
