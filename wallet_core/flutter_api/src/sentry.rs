pub use std::borrow::Cow;

use once_cell::sync::OnceCell;
pub use sentry::{init, release_name, ClientInitGuard, ClientOptions};

static SENTRY: OnceCell<Option<ClientInitGuard>> = OnceCell::new();

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
