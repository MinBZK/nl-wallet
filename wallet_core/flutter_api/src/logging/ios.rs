use std::io::{Result, Write};

use oslog::OsLog;
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Default)]
pub struct WriterMaker();

/// We need something that implements the [`WriterMaker`] trait in order to have different
/// [`LogWriter`] instances per debug level.
///
/// * For [`Level::ERROR`] messages, use the fault logging function.
/// * For [`Level::WARN`] messages. use the error logging function.
/// * For any message that is [`Level::INFO`] or below, use the default logging function.
///   This is necessary, because Flutter will not show output on the console for the info
///   and debug logging functions.
impl<'a> MakeWriter<'a> for WriterMaker {
    type Writer = LogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        LogWriter::default()
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        match *meta.level() {
            Level::ERROR => LogWriter::fault(),
            Level::WARN => LogWriter::error(),
            _ => LogWriter::default(),
        }
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

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let fun = self.fun;
        fun(&self.log, String::from_utf8_lossy(buf).as_ref());

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
