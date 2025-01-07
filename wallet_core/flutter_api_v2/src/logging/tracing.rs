use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;

pub fn init_tracing_subscriber() {
    let builder = SubscriberBuilder::default();

    // For release builds, set the log level to WARN and remove timestamps.
    #[cfg(not(debug_assertions))]
    let builder = builder.with_max_level(Level::WARN).without_time();

    // For debug builds, set the log level to DEBUG.
    #[cfg(debug_assertions)]
    let builder = builder.with_max_level(Level::DEBUG);

    // For Android and iOS, use a custom log writer instead of stdout.
    #[cfg(target_os = "android")]
    type WriterMaker = super::android::WriterMaker;

    #[cfg(target_os = "ios")]
    type WriterMaker = super::ios::WriterMaker;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let builder = builder.with_writer(WriterMaker::default());

    // Set the result of the builder as global default subscriber.
    builder.init();
}
