use std::error::Error;

use sentry::parse_type_from_debug;
use sentry::protocol::Event;
use sentry::protocol::Exception;
use sentry::Level;

use crate::Category;
use crate::ErrorCategory;

/// Create a sentry [`Event`] from an [`ErrorCategory`].
/// A tag `category` with the string representation of the [`ErrorCategory`] is added to the event, so
/// the `filter_and_scrub_sensitive_data` can act according to the category of the error.
///
/// For errors that fall into the category `unexpected` an error message is logged.
/// Unexpected errors should never occur in the wallet and point to a programming error, this can happen when
/// the wallet uses code that is meant for an external service, like the wallet_provider or the wallet_server.
/// Otherwise the error classification is wrong.
pub fn classify_mask_and_capture<T: ErrorCategory + Error + ?Sized>(error: &T) {
    let category = error.category();
    if category == Category::Unexpected {
        tracing::error!("unexpected error, this should never occur in the Wallet: {error}");
    }
    let mut event = event_from_error(error);
    event
        .tags
        .insert("category".to_owned(), format!("{category}").to_owned());
    sentry::capture_event(event);
}

/// Filter and scrub events
pub fn filter_and_scrub_sensitive_data(mut event: Event) -> Option<Event> {
    let category: Option<Category> = event.tags.get("category").and_then(|t| t.parse().ok());
    match category {
        Some(Category::Unexpected) => {
            tracing::error!(
                "event has category unexpected, this is a programming error, sending scrubbed event to Sentry"
            );
            event.scrub(true);
            Some(event)
        }
        Some(Category::Expected) => {
            tracing::debug!("event has category expected, not sending to Sentry");
            None
        }
        Some(Category::Critical) => {
            tracing::debug!("event has category critical, sending event to Sentry verbatim");
            event.scrub(false);
            Some(event)
        }
        Some(Category::PersonalData) => {
            tracing::debug!("event has category pd, sending scrubbed event to Sentry");
            event.scrub(true);
            Some(event)
        }
        None => {
            tracing::debug!("uncategorized event, sending scrubbed event to Sentry");
            event.scrub(true);
            Some(event)
        }
    }
}

/// Scrub sensitive data.
/// By default breadcrumb, transaction and request data is removed as these might be filled automatically by
/// the `sentry` crate and/or some of its integrations.
/// If `sensitive_messages` is true, the value of exception messages is removed, these contain the error
/// descriptions according to the `Display` implementation of each `Error`.
trait Scrub {
    fn scrub(&mut self, sensitive_messages: bool);
}

/// Scrubbing sensitive data from [`Event`] according to:
/// https://docs.sentry.io/platforms/rust/data-management/sensitive-data/#scrubbing-data.
///
/// According to the docs, in general sensitive data can appear in the following areas:
/// - stacktraces: Rust stacktraces do not contain sensitive data
/// - breadcrumbs: By default Rust does not fill breadcrumbs, although it can be done manually or via logging/tracing
/// - contextual information: Inspection showed no sensitive information, the following are detected for the Wallet
///   - device: arch
///   - os: name, version, kernel_version
///   - rust: name, version, channel
/// - transactional data: Rust can fill this for tower services, this is not configured for the Wallet
/// - request: Rust can fill this for tower services, this is not configured for the Wallet
impl Scrub for Event<'_> {
    fn scrub(&mut self, sensitive_messages: bool) {
        self.breadcrumbs = Default::default();
        self.transaction = None;
        self.request = None;

        self.exception.iter_mut().for_each(|e| e.scrub(sensitive_messages));
    }
}

/// Scrub `value` from [`Exception`] since it might contain sensitive data
impl Scrub for Exception {
    fn scrub(&mut self, sensitive_messages: bool) {
        if sensitive_messages {
            self.value = None;
        }
    }
}

/// Create a sentry `Event` from a `std::error::Error`.
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
/// let event = error_category::sentry::event_from_error(&OuterError(InnerError));
/// assert_eq!(event.level, sentry::protocol::Level::Error);
/// assert_eq!(event.exception.len(), 2);
/// assert_eq!(&event.exception[0].ty, "InnerError");
/// assert_eq!(event.exception[0].value, Some("inner".to_string()));
/// assert!(&event.exception[1].ty.ends_with("::OuterError"));
/// assert_eq!(event.exception[1].value, Some("outer".to_string()));
/// ```
///
/// [sentry event payloads]: https://develop.sentry.dev/sdk/event-payloads/exception/
pub fn event_from_error<E: Error + ?Sized>(err: &E) -> Event<'static> {
    let mut exceptions = vec![exception_from_error(err)];

    let mut source = err.source();
    while let Some(err) = source {
        exceptions.push(exception_from_error(err));
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
/// This version provides better type information for the root error in the chain.
fn exception_from_error<E: Error + ?Sized>(err: &E) -> Exception {
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
        value: Some(value),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use sentry::test::with_captured_events;
    use sentry::Level;
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

    #[rstest]
    #[case(Category::PersonalData, "pd")]
    #[case(Category::Critical, "critical")]
    #[case(Category::Expected, "expected")]
    #[case(Category::Unexpected, "unexpected")]
    fn test_classify_mask_and_capture_enum(#[case] category: Category, #[case] expected_tag: String) {
        let error = ErrorEnum::Specific(SpecificError { category });
        let mut events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 1);
        let event = events.remove(0);
        assert_eq!(event.level, Level::Error);
        assert_eq!(event.exception.values.len(), 2);
        assert_eq!(event.exception.values[0].ty, "SpecificError".to_string());
        assert_eq!(
            event.exception.values[1].ty,
            "error_category::sentry::tests::ErrorEnum::Specific".to_string()
        );
        assert_eq!(event.exception.values[0].value, Some(ERROR_MSG.to_string()));
        assert!(format!("{:?}", event).contains(ERROR_MSG));
        let category = event.tags.get("category");
        assert_eq!(category, Some(&expected_tag));
    }

    #[rstest]
    #[case(Category::PersonalData, "pd")]
    #[case(Category::Critical, "critical")]
    #[case(Category::Expected, "expected")]
    #[case(Category::Unexpected, "unexpected")]
    fn test_classify_mask_and_capture_critical_struct(#[case] category: Category, #[case] expected_tag: String) {
        let error = SpecificError { category };
        let mut events = with_captured_events(|| {
            classify_mask_and_capture(&error);
        });
        assert_eq!(events.len(), 1);
        let event = events.remove(0);
        assert_eq!(event.level, Level::Error);
        assert_eq!(event.exception.values.len(), 1);
        assert_eq!(
            event.exception.values[0].ty,
            "error_category::sentry::tests::SpecificError".to_string()
        );
        assert_eq!(event.exception.values[0].value, Some(ERROR_MSG.to_string()));
        assert!(format!("{:?}", event).contains(ERROR_MSG));
        let category = event.tags.get("category");
        assert_eq!(category, Some(&expected_tag));
    }
}
