pub use std::borrow::Cow;
use std::sync::Arc;
use std::sync::OnceLock;

pub use error_category::sentry::filter_and_scrub_sensitive_data;
pub use sentry::ClientInitGuard;
pub use sentry::ClientOptions;
pub use sentry::init;
pub use sentry::release_name;

static SENTRY: OnceLock<Option<ClientInitGuard>> = OnceLock::new();
const MAX_BREADCRUMBS: usize = 25;

pub(crate) fn allow_logs() -> bool {
    allow_logs_for_env(option_env!("ALLOW_RELEASE_LOGS"), cfg!(debug_assertions))
}

pub(crate) fn allow_logs_for_env(value: Option<&str>, debug_assertions: bool) -> bool {
    debug_assertions || allow_release_logs_for_env(value)
}

fn allow_release_logs_for_env(value: Option<&str>) -> bool {
    value == Some("true")
}

pub(crate) fn init_sentry() {
    let _ = SENTRY.get_or_init(|| {
        option_env!("SENTRY_DSN").filter(|dsn| !dsn.is_empty()).map(|dsn| {
            init((
                dsn,
                ClientOptions {
                    release: option_env!("SENTRY_RELEASE")
                        .filter(|release| !release.is_empty())
                        .map(Cow::from)
                        .or_else(|| release_name!()),
                    environment: option_env!("SENTRY_ENVIRONMENT")
                        .filter(|environment| !environment.is_empty())
                        .map(Cow::from),
                    send_default_pii: false,
                    max_breadcrumbs: MAX_BREADCRUMBS,
                    debug: cfg!(debug_assertions),
                    enable_logs: allow_logs(),
                    before_send: Some(Arc::new(filter_and_scrub_sensitive_data)),
                    ..Default::default()
                },
            ))
        })
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
