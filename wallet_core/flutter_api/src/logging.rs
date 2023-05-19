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

    // For Android and iOS, disable ANSI colours and use a custom log writer instead of stdout.
    #[cfg(target_os = "android")]
    let builder = builder.with_ansi(false).with_writer(android::WriterMaker::default());

    #[cfg(target_os = "ios")]
    let builder = builder.with_ansi(false).with_writer(ios::LogWriter::default);

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

#[cfg(target_os = "android")]
mod android {
    use std::{ffi::CStr, fmt::Write, io};

    use android_logger::{LogId, PlatformLogWriter};
    use cstr::cstr;
    use tracing::{Level, Metadata};
    use tracing_subscriber::fmt::MakeWriter;

    /// Print "core" as the tag in Android logs, to differentiate from Flutter messages.
    const TAG: &CStr = cstr!("core");
    const DEFAULT_TRACE_LEVEL: &Level = &Level::INFO;

    /// We need something that implements the [`WriterMaker`] trait in order to have different
    /// [`LogWriter`] instances per debug level.
    #[derive(Default)]
    pub struct WriterMaker();

    impl WriterMaker {
        fn writer(&self, level: &Level) -> LogWriter {
            let level = match *level {
                Level::TRACE => log::Level::Trace,
                Level::DEBUG => log::Level::Debug,
                Level::INFO => log::Level::Info,
                Level::WARN => log::Level::Warn,
                Level::ERROR => log::Level::Error,
            };

            LogWriter(PlatformLogWriter::new(Some(LogId::Main), level, TAG))
        }
    }

    impl<'a> MakeWriter<'a> for WriterMaker {
        type Writer = LogWriter<'a>;

        fn make_writer(&'a self) -> Self::Writer {
            // This method may never get called (as there should normally be metadata present),
            // but if it does we should just pick a debug level ourselves.
            self.writer(DEFAULT_TRACE_LEVEL)
        }

        fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
            self.writer(meta.level())
        }
    }

    /// This wraps an instance of [`PlatformLogWriter`], which is a low-level type
    /// contained in the [`android_logger`] crate. We use this so we can more directly
    /// write to the Android logger, instead of using the higher-level components offered
    /// by [`android_logger`].
    ///
    /// Unfortunately, [`PlatformLogWriter`] implements the [`std::fmt::Write`] trait,
    /// instead of the [`std::io::Write`] trait that is required, so have have to convert
    /// between the two. In practice this means converting the provided by slices back to
    /// strings.
    pub struct LogWriter<'a>(PlatformLogWriter<'a>);

    impl io::Write for LogWriter<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0
                .write_str(&String::from_utf8_lossy(buf))
                .map(|_| buf.len())
                // Convert any resulting error to io::Error.
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            // Event though we implement flush() below, this does not seem to get called!
            // For that reason we just flush after every write, so the tracing statements
            // actually show up in the Android logs.
            self.0.flush();

            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush();

            Ok(())
        }
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use std::io::{Result, Write};

    use oslog::OsLog;

    /// This wraps a OSLog instance so it can implement [`Write`].
    pub struct LogWriter(OsLog);

    impl From<OsLog> for LogWriter {
        fn from(value: OsLog) -> Self {
            LogWriter(value)
        }
    }

    /// The default instance simply wraps the global OSLog.
    impl Default for LogWriter {
        fn default() -> Self {
            LogWriter(OsLog::global())
        }
    }

    impl Write for LogWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
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
