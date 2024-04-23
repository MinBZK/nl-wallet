pub use std::borrow::Cow;

pub use sentry::ClientInitGuard;

pub(crate) fn init_sentry() -> Option<ClientInitGuard> {
    option_env!("SENTRY_DSN").map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: option_env!("SENTRY_ENVIRONMENT").map(Cow::from),
                debug: cfg!(debug_assertions),
                attach_stacktrace: true,
                ..Default::default()
            },
        ))
    })
}
