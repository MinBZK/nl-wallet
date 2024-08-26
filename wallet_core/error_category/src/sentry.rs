use std::error::Error;

use sentry::{
    parse_type_from_debug,
    protocol::{Event, Exception},
    Level,
};
use tracing::debug;

use crate::{Category, ErrorCategory};

pub fn classify_mask_and_capture<T: ErrorCategory + Error + ?Sized>(error: &T) {
    match error.category() {
        Category::Expected => {
            debug!("encountered expected error, not reporting to sentry: {}", error);
        }
        Category::Critical => {
            debug!("encountered critical error, reporting to sentry: {}", error);
            let event = event_from_error(error, false);
            let _uuid = sentry::capture_event(event);
        }
        Category::PersonalData => {
            debug!(
                "encountered critical error with possible PII, reporting to sentry without data: {}",
                error
            );
            let event = event_from_error(error, true);
            let _uuid = sentry::capture_event(event);
        }
        Category::Unexpected => {
            panic!(
                "encountered unexpected error, which means that it should never occur in the Wallet: {}",
                error
            );
        }
    }
}

/// Create a sentry `Event` from a `std::error::Error`, remove PII by setting `sensitive` to true.
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
/// let event = error_category::sentry::event_from_error(&OuterError(InnerError), true);
/// assert_eq!(event.level, sentry::protocol::Level::Error);
/// assert_eq!(event.exception.len(), 2);
/// assert_eq!(&event.exception[0].ty, "InnerError");
/// assert_eq!(event.exception[0].value, None);
/// assert!(&event.exception[1].ty.ends_with("::OuterError"));
/// assert_eq!(event.exception[1].value, None);
/// ```
///
/// [sentry event payloads]: https://develop.sentry.dev/sdk/event-payloads/exception/
pub fn event_from_error<E: Error + ?Sized>(err: &E, sensitive: bool) -> Event<'static> {
    let mut exceptions = vec![exception_from_error(err, sensitive)];

    let mut source = err.source();
    while let Some(err) = source {
        exceptions.push(exception_from_error(err, sensitive));
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
/// Extended with `sensitive` argument, to allow for removing sensitive data.
fn exception_from_error<E: Error + ?Sized>(err: &E, sensitive: bool) -> Exception {
    let dbg = format!("{err:?}");
    let value = err.to_string();
    let type_name = std::any::type_name::<E>();

    // Determine type identifier to use in the Sentry event.
    let ty = if type_name == "dyn core::error::Error" {
        // `std::any::type_name` will only work successfully for the root error in the
        // chain, because the compiler loses type information as `err.source()` is used
        // to iterate the chain. In case `type_name == dyn std::error::Error`, we will
        // not use it in the type identifier.
        parse_type_from_debug(&dbg).to_owned()
    } else if dbg == format!("{value:?}") {
        // A generic `anyhow::msg` will just `Debug::fmt` the `String` that you feed
        // it. Trying to parse the type name from that will result in a leading quote
        // and the first word, so quite useless.
        // To work around this, we check if the `Debug::fmt` of the complete error
        // matches its `Display::fmt`, in which case there is no type to parse and
        // we will be using `type_name::<E>`.

        // Here we diverge from the original which returns "Error"
        type_name.to_owned()
    } else {
        let variant = parse_type_from_debug(&dbg);
        // For a struct, `variant` will be the type name, so if the fully qualified
        // `type_name` ends with `variant`, we will just use `type_name` as type
        // identifier.
        if type_name.ends_with(variant) {
            type_name.to_owned()
        } else {
            format!("{}::{}", type_name, variant)
        }
    };
    Exception {
        ty,
        value: (!sensitive).then_some(value),
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
    fn test_classify_mask_and_capture_expected() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::Expected,
        });
        let events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 0);
    }

    #[test]
    #[should_panic(
        expected = "encountered unexpected error, which means that it should never occur in the Wallet: Some error: \
                    My error message"
    )]
    fn test_classify_mask_and_capture_unexpected() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::Unexpected,
        });
        classify_mask_and_capture(&error);
    }

    #[test]
    fn test_classify_mask_and_capture_critical() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::Critical,
        });
        let events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values.len(), 2);
        assert_eq!(events[0].exception.values[0].ty, "SpecificError".to_string());
        assert_eq!(
            events[0].exception.values[1].ty,
            "error_category::sentry::tests::ErrorEnum::Specific".to_string()
        );
        assert_eq!(events[0].exception.values[0].value, Some(ERROR_MSG.to_string()));
        assert!(format!("{:?}", events[0]).contains(ERROR_MSG));
    }

    #[test]
    fn test_classify_mask_and_capture_critical_struct() {
        let error = SpecificError {
            category: Category::Critical,
        };
        let events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values.len(), 1);
        assert_eq!(
            events[0].exception.values[0].ty,
            "error_category::sentry::tests::SpecificError".to_string()
        );
        assert_eq!(events[0].exception.values[0].value, Some(ERROR_MSG.to_string()));
        assert!(format!("{:?}", events[0]).contains(ERROR_MSG));
    }

    #[test]
    fn test_classify_mask_and_capture_personal_data() {
        let error = ErrorEnum::Specific(SpecificError {
            category: Category::PersonalData,
        });
        let events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::Error);
        assert_eq!(events[0].exception.values[0].ty, "SpecificError".to_string());
        assert_eq!(
            events[0].exception.values[1].ty,
            "error_category::sentry::tests::ErrorEnum::Specific".to_string()
        );
        assert_eq!(events[0].exception.values[0].value, None);
        assert_eq!(events[0].exception.values[1].value, None);
        assert!(!format!("{:?}", events[0]).contains(ERROR_MSG));
    }
}
