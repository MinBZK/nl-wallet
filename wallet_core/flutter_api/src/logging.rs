use std::sync::Once;

use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;

static LOGGING: Once = Once::new();

pub fn init_logging() {
    // Make sure this initializer can be called multiple times, but executes only once.
    LOGGING.call_once(|| {
        // For release builds, set the log level to ERROR.
        let builder = SubscriberBuilder::default().with_max_level(Level::ERROR);

        // For debug builds, set the log level to DEBUG.
        #[cfg(debug_assertions)]
        let builder = builder.with_max_level(Level::DEBUG);

        // For iOS, disable ANSI colours and use a custom log writer instead of stdout.
        #[cfg(target_os = "ios")]
        let builder = builder.with_ansi(false).with_writer(ios::OsLogWriter::default);

        // Set the result of the builder as global default subscriber.
        builder.init();
    })
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
