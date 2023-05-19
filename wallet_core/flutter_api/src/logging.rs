use std::sync::Once;

use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;

static LOGGING: Once = Once::new();

pub fn init_logging() {
    // Make sure this initializer can be called multiple times, but executes only once.
    LOGGING.call_once(|| {
        // Set up a subscriber to log to the relevant output for the platform.
        init_tracing_subscriber();

        // Set a custom panic handler for debug builds. For release builds
        // we can rely on Flutter logging panics as uncaught exceptions.
        // As init_tracing_subscriber() might panic, we want that to be caught by
        // the default handler, as we cannot log anything yet anyway.
        #[cfg(debug_assertions)]
        init_panic_logger();
    });
}

fn init_tracing_subscriber() {
    let builder = SubscriberBuilder::default();

    // For release builds, set the log level to WARN and remove timestamps.
    #[cfg(not(debug_assertions))]
    let builder = builder.with_max_level(Level::WARN).without_time();

    // For debug builds, set the log level to DEBUG.
    #[cfg(debug_assertions)]
    let builder = builder.with_max_level(Level::DEBUG);

    // For iOS, disable ANSI colours and use a custom log writer instead of stdout.
    #[cfg(target_os = "ios")]
    let builder = builder.with_ansi(false).with_writer(ios::OsLogWriter::default);

    // Set the result of the builder as global default subscriber.
    builder.init();
}

#[cfg(debug_assertions)]
fn init_panic_logger() {
    use std::{backtrace::Backtrace, panic};

    use tracing::error;

    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::force_capture();

        // The payload may either be a reference to a [`String`] or a `&'static str`.
        let payload = panic_info.payload();
        let message = match (payload.downcast_ref::<String>(), payload.downcast_ref::<&'static str>()) {
            (Some(s), _) => Some(s.as_ref()),
            (_, Some(s)) => Some(*s),
            (_, _) => None,
        };

        // Log the panic message and backtrace, each on separate lines
        // because OSLog on iOS has a 1024 character limit.
        // See: https://stackoverflow.com/questions/39584707/nslog-on-devices-in-ios-10-xcode-8-seems-to-truncate-why/40283623#40283623
        //
        // Note that we need to use string formatting to prevent
        // the [`error!`] macro from printing the variable name.
        error!("Panic occurred: {}", message.unwrap_or("UNKNOWN"));
        backtrace
            .to_string()
            .split('\n')
            .filter(|backtrace_line| !backtrace_line.is_empty())
            .for_each(|backtrace_line| error!("{}", backtrace_line));
    }));
}

#[cfg(target_os = "ios")]
mod ios {
    use std::io::Write;

    use oslog::OsLog;

    /// This wraps a OSLog instance so it can implement [`Write`].
    pub struct OsLogWriter(OsLog);

    impl From<OsLog> for OsLogWriter {
        fn from(value: OsLog) -> Self {
            OsLogWriter(value)
        }
    }

    /// The default instance simply wraps the global OSLog.
    impl Default for OsLogWriter {
        fn default() -> Self {
            OsLogWriter(OsLog::global())
        }
    }

    impl Write for OsLogWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            // Do not actually use differentiated logging levels of OSLog, since the Flutter
            // console only outputs the default level or higher (i.e. not info or debug).
            self.0.default(String::from_utf8_lossy(buf).as_ref());

            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
