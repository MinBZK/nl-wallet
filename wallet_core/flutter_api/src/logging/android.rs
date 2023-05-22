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
