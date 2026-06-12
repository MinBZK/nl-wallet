pub use std::borrow::Cow;
use std::sync::Arc;
use std::sync::OnceLock;

pub use error_category::sentry::filter_and_scrub_sensitive_data;
pub use sentry::ClientInitGuard;
pub use sentry::ClientOptions;
pub use sentry::init;
pub use sentry::release_name;

static SENTRY: OnceLock<Option<ClientInitGuard>> = OnceLock::new();

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
                    debug: cfg!(debug_assertions),
                    before_send: Some(Arc::new(filter_and_scrub_sensitive_data)),
                    ..Default::default()
                },
            ))
        })
    });
}
