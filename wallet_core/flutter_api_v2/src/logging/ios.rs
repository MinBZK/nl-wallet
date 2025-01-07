use std::io::Result;
use std::io::Write;

use oslog::OsLog;
use tracing::Level;
use tracing::Metadata;
use tracing_subscriber::fmt::MakeWriter;

const DEFAULT_LEVEL: &tracing::Level = &tracing::Level::INFO;

/// We need something that implements the [`WriterMaker`] trait in order to have different
/// [`LogWriter`] instances per debug level.
///
/// * For [`Level::ERROR`] messages, use the fault logging function.
/// * For [`Level::WARN`] messages. use the error logging function.
/// * For any message that is [`Level::INFO`] or below, use the default logging function. This is necessary, because
///   Flutter will not show output on the console for the info and debug logging functions.
pub struct WriterMaker {
    default_writer: LogWriter,
    error_writer: LogWriter,
    fault_writer: LogWriter,
}

impl WriterMaker {
    fn new() -> Self {
        WriterMaker {
            default_writer: LogWriter::default(),
            error_writer: LogWriter::error(),
            fault_writer: LogWriter::fault(),
        }
    }

    /// Map the tracing level to the writer with the appropriate log level.
    fn writer(&self, level: &Level) -> &LogWriter {
        match *level {
            Level::ERROR => &self.fault_writer,
            Level::WARN => &self.error_writer,
            _ => &self.default_writer,
        }
    }
}

impl Default for WriterMaker {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MakeWriter<'a> for WriterMaker {
    type Writer = &'a LogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        // This method may never get called (as there should normally be metadata present),
        // but if it does we should just return the default writer.
        self.writer(DEFAULT_LEVEL)
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        self.writer(meta.level())
    }
}

/// This wraps a OSLog instance so it can implement [`Write`].
pub struct LogWriter {
    log: OsLog,
    fun: fn(&OsLog, &str),
}

impl LogWriter {
    /// Wrap the global OSLog with a specific logging function.
    fn global(fun: fn(&OsLog, &str)) -> Self {
        LogWriter {
            log: OsLog::global(),
            fun,
        }
    }

    /// Wrap the global OSLog and the error logging function.
    fn error() -> Self {
        Self::global(OsLog::error)
    }

    /// Wrap the global OSLog and the fault logging function.
    fn fault() -> Self {
        Self::global(OsLog::fault)
    }
}

/// The default instance simply wraps the global OSLog and the default logging function.
impl Default for LogWriter {
    fn default() -> Self {
        Self::global(OsLog::default)
    }
}

impl Write for &LogWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (self.fun)(&self.log, String::from_utf8_lossy(buf).as_ref());

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
