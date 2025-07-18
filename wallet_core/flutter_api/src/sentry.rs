pub use std::borrow::Cow;
use std::sync::Arc;
use std::sync::OnceLock;

pub use sentry::ClientInitGuard;
pub use sentry::ClientOptions;
pub use sentry::init;
pub use sentry::release_name;

pub use error_category::sentry::filter_and_scrub_sensitive_data;

static SENTRY: OnceLock<Option<ClientInitGuard>> = OnceLock::new();

pub(crate) fn init_sentry() {
    let _ = SENTRY.get_or_init(|| {
        option_env!("SENTRY_DSN").map(|dsn| {
            init((
                dsn,
                ClientOptions {
                    release: release_name!(),
                    environment: option_env!("SENTRY_ENVIRONMENT").map(Cow::from),
                    debug: cfg!(debug_assertions),
                    before_send: Some(Arc::new(filter_and_scrub_sensitive_data)),
                    ..Default::default()
                },
            ))
        })
    });
}
