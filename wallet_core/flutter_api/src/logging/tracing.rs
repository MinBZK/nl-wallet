use sentry::integrations::tracing::EventFilter;
use tracing::Level;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::sentry::allow_logs;

pub fn init_tracing_subscriber() {
    let logs_allowed = allow_logs();

    let max_level = if logs_allowed {
        LevelFilter::DEBUG
    } else {
        LevelFilter::OFF
    };

    // Timestamps are convenient when running with `flutter run`; release builds omit them below.
    let fmt_layer = fmt::layer();

    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt_layer.without_time();

    // For Android and iOS, use a custom log writer instead of stdout.
    #[cfg(target_os = "android")]
    type WriterMaker = super::android::WriterMaker;

    #[cfg(target_os = "ios")]
    type WriterMaker = super::ios::WriterMaker;

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let fmt_layer = fmt_layer.with_writer(WriterMaker::default());

    let fmt_layer = fmt_layer.with_filter(max_level);

    let registry = tracing_subscriber::registry().with(fmt_layer);

    // Set the result of the builder as global default subscriber.
    if logs_allowed {
        registry
            .with(
                sentry::integrations::tracing::layer().event_filter(|metadata| match *metadata.level() {
                    Level::ERROR | Level::WARN | Level::INFO | Level::DEBUG => EventFilter::Log,
                    Level::TRACE => EventFilter::Ignore,
                }),
            )
            .init();
    } else {
        registry.init();
    }
}
