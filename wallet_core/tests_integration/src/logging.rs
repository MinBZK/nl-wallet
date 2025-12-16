use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with_test_writer()
        .init();
}
