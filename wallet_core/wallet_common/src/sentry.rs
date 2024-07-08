use std::{borrow::Cow, error::Error};

use sentry::{
    parse_type_from_debug,
    protocol::{Event, Exception},
    ClientInitGuard, ClientOptions, Level,
};
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

pub fn classify_and_report_to_sentry<T: ErrorCategory + Error>(error: &T) {
    match error.category() {
        Category::Expected => {}
        Category::Critical => {
            let _uuid = sentry::capture_error(error);
        }
        Category::PersonalData => {
            let event = privacy_sensitive_event_from_error(error);
            let _uuid = sentry::capture_event(event);
        }
    }
}

/// Create a sentry `Event` from a `std::error::Error`, without any PII data.
///
/// Copied from `sentry::event_from_error`.
///
/// A chain of errors will be resolved as well, and sorted oldest to newest, as
/// described in the [sentry event payloads].
///
/// # Examples
///
/// ```
/// use thiserror::Error;
///
/// #[derive(Debug, Error)]
/// #[error("inner")]
/// struct InnerError;
///
/// #[derive(Debug, Error)]
/// #[error("outer")]
/// struct OuterError(#[from] InnerError);
///
/// let event = sentry::privacy_sensitive_event_from_error(&OuterError(InnerError));
/// assert_eq!(event.level, sentry::protocol::Level::Error);
/// assert_eq!(event.exception.len(), 2);
/// assert_eq!(&event.exception[0].ty, "InnerError");
/// assert_eq!(event.exception[0].value, None);
/// assert_eq!(&event.exception[1].ty, "OuterError");
/// assert_eq!(event.exception[1].value, None);
/// ```
///
/// [sentry event payloads]: https://develop.sentry.dev/sdk/event-payloads/exception/
fn privacy_sensitive_event_from_error<E: Error + ?Sized>(err: &E) -> Event<'static> {
    let mut exceptions = vec![privacy_sensitive_exception_from_error(err)];

    let mut source = err.source();
    while let Some(err) = source {
        exceptions.push(privacy_sensitive_exception_from_error(err));
        source = err.source();
    }

    exceptions.reverse();
    Event {
        exception: exceptions.into(),
        level: Level::Error,
        ..Default::default()
    }
}

/// Copied from `sentry::exception_from_error`.
fn privacy_sensitive_exception_from_error<E: Error + ?Sized>(err: &E) -> Exception {
    let dbg = format!("{err:?}");
    let value = err.to_string();

    // A generic `anyhow::msg` will just `Debug::fmt` the `String` that you feed
    // it. Trying to parse the type name from that will result in a leading quote
    // and the first word, so quite useless.
    // To work around this, we check if the `Debug::fmt` of the complete error
    // matches its `Display::fmt`, in which case there is no type to parse and
    // we will be using `type_name::<E>`.
    let ty = if dbg == format!("{value:?}") {
        // Here we diverge from the original which returns "Error"
        std::any::type_name::<E>().to_string()
    } else {
        parse_type_from_debug(&dbg).to_owned()
    };
    Exception {
        ty,
        value: None,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use sentry::{test::with_captured_events, Level};
    use thiserror::Error;

    use super::*;

    const ERROR_MSG: &str = "My error message";

    #[derive(Debug, Error)]
    #[error("My error message")]
    struct SpecificError {
        category: Category,
    }

    impl ErrorCategory for SpecificError {
        fn category(&self) -> Category {
            self.category
        }
    }

    #[derive(Debug, Error)]
    enum ErrorEnum {
        #[error("Some error: {0}")]
        Specific(#[from] SpecificError),
    }

    impl ErrorCategory for ErrorEnum {
        fn category(&self) -> Category {
            match self {
                Self::Specific(error) => error.category(),
            }
        }
    }

    #[test]
    fn test_classify_and_report_to_sentry_expected() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::Expected,
        });
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(&error);
        });
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_classify_and_report_to_sentry_critical() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::Critical,
        });
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(&error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values.len(), 2);
        assert_eq!(events[0].exception.values[0].ty, "SpecificError".to_string());
        assert_eq!(events[0].exception.values[1].ty, "Specific".to_string());
        assert_eq!(events[0].exception.values[0].value, Some(ERROR_MSG.to_string()));
        assert!(format!("{:?}", events[0]).contains(ERROR_MSG));
    }

    #[test]
    fn test_classify_and_report_to_sentry_personal_data() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::PersonalData,
        });
        let events = with_captured_events(|| {
            classify_and_report_to_sentry(&error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values[0].ty, "SpecificError".to_string());
        assert_eq!(events[0].exception.values[1].ty, "Specific".to_string());
        assert_eq!(events[0].exception.values[0].value, None);
        assert_eq!(events[0].exception.values[1].value, None);
        assert!(!format!("{:?}", events[0]).contains(ERROR_MSG));
    }
}
