use std::{
    ffi::CStr,
    fmt::Write,
    io,
    sync::{Mutex, MutexGuard},
};

use android_logger::{LogId, PlatformLogWriter};
use cstr::cstr;
use tracing::{log, Metadata};
use tracing_subscriber::fmt::MakeWriter;

/// Print "core" as the tag in Android logs, to differentiate from Flutter messages.
const TAG: &CStr = cstr!("core");
const DEFAULT_LEVEL: &tracing::Level = &tracing::Level::INFO;

type PlatformLogWriterMutex = Mutex<PlatformLogWriter<'static>>;

fn platform_log_writer_mutex(level: log::Level) -> PlatformLogWriterMutex {
    Mutex::new(PlatformLogWriter::new(Some(LogId::Main), level, TAG))
}

/// We need something that implements the [`WriterMaker`] trait in order to have different
/// [`LogWriter`] instances per debug level.
pub struct WriterMaker {
    trace_writer: PlatformLogWriterMutex,
    debug_writer: PlatformLogWriterMutex,
    info_writer: PlatformLogWriterMutex,
    warn_writer: PlatformLogWriterMutex,
    error_writer: PlatformLogWriterMutex,
}

impl WriterMaker {
    fn new() -> Self {
        WriterMaker {
            trace_writer: platform_log_writer_mutex(log::Level::Debug),
            debug_writer: platform_log_writer_mutex(log::Level::Debug),
            info_writer: platform_log_writer_mutex(log::Level::Info),
            warn_writer: platform_log_writer_mutex(log::Level::Warn),
            error_writer: platform_log_writer_mutex(log::Level::Error),
        }
    }

    /// Map the tracing level to a writer with the appropriate log level.
    fn writer<'a>(&'a self, level: &tracing::Level) -> LogWriter<'a, 'static> {
        let writer = match *level {
            tracing::Level::TRACE => &self.trace_writer,
            tracing::Level::DEBUG => &self.debug_writer,
            tracing::Level::INFO => &self.info_writer,
            tracing::Level::WARN => &self.warn_writer,
            tracing::Level::ERROR => &self.error_writer,
        };

        LogWriter(writer.lock().unwrap())
    }
}

impl Default for WriterMaker {
    fn default() -> Self {
        WriterMaker::new()
    }
}

impl<'a> MakeWriter<'a> for WriterMaker {
    type Writer = LogWriter<'a, 'static>;

    fn make_writer(&'a self) -> Self::Writer {
        // This method may never get called (as there should normally be metadata present),
        // but if it does we should just pick a debug level ourselves.
        self.writer(DEFAULT_LEVEL)
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        self.writer(meta.level())
    }
}

/// This wraps a [`MutexGuard`], which in turn wraps an instance of [`PlatformLogWriter`],
/// which is a low-level type contained in the [`android_logger`] crate. We use this
/// so we can more directly write to the Android logger, instead of using the higher-level
/// components provided by [`android_logger`]. The rationale for the [`MutexGuard`] is
/// that the [`MakeWriter`] trait implemented by [`WriterMaker`] does take `&mut self`
/// as an argument to its methods.
///
/// Unfortunately, [`PlatformLogWriter`] implements the [`std::fmt::Write`] trait,
/// instead of the [`std::io::Write`] trait that is required, so have have to convert
/// between the two. In practice this means converting the provided by slices back to
/// strings.
pub struct LogWriter<'a, 'b>(MutexGuard<'a, PlatformLogWriter<'b>>);

impl io::Write for LogWriter<'_, '_> {
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
