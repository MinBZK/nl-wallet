mod tracing;

#[cfg(debug_assertions)]
mod panic;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

use parking_lot::Once;

use self::tracing::init_tracing_subscriber;

static LOGGING: Once = Once::new();

pub(crate) fn allow_logs() -> bool {
    allow_logs_for_env(option_env!("ALLOW_RELEASE_LOGS"), cfg!(debug_assertions))
}

pub(crate) fn allow_logs_for_env(value: Option<&str>, debug_assertions: bool) -> bool {
    debug_assertions || allow_release_logs_for_env(value)
}

fn allow_release_logs_for_env(value: Option<&str>) -> bool {
    value == Some("true")
}

pub fn init_logging() {
    // Make sure this initializer can be called multiple times, but executes only once.
    LOGGING.call_once(|| {
        // Set up a subscriber to log to the relevant output for the platform.
        init_tracing_subscriber();

        // Set a custom panic handler for debug builds. For release builds
        // we can rely on Flutter logging panics as uncaught exceptions.
        // As init_tracing_subscriber() might panic, we want that to be caught by
        // the default handler, as we cannot log anything yet anyway.
        #[cfg(debug_assertions)]
        self::panic::init_panic_logger();
    });
}

#[cfg(test)]
mod tests {
    use super::allow_logs_for_env;

    #[test]
    fn logs_are_enabled_for_debug_builds_by_default() {
        assert!(allow_logs_for_env(None, true));
    }

    #[test]
    fn release_logs_are_disabled_by_default() {
        assert!(!allow_logs_for_env(None, false));
    }

    #[test]
    fn release_logs_can_be_enabled_explicitly() {
        assert!(allow_logs_for_env(Some("true"), false));

        assert!(!allow_logs_for_env(Some("false"), false));
        assert!(!allow_logs_for_env(Some(""), false));
        assert!(!allow_logs_for_env(Some("TRUE"), false));
        assert!(!allow_logs_for_env(Some("1"), false));
    }
}
