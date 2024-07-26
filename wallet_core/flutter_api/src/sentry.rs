pub use std::borrow::Cow;
use std::sync::OnceLock;

pub use sentry::{init, release_name, ClientInitGuard, ClientOptions};

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
                    ..Default::default()
                },
            ))
        })
    });
}
