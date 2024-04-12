pub use sentry::ClientInitGuard;

pub(crate) fn init_sentry() -> Option<ClientInitGuard> {
    option_env!("SENTRY_DSN").map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                attach_stacktrace: true,
                traces_sample_rate: 1.0,
                debug: true,
                ..Default::default()
            },
        ))
    })
}
