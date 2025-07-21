use std::fmt::Debug;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::os::fd::RawFd;
use std::thread;

use libc;
use log::Level;
use regex::Regex;

struct Pipe {
    read_fd: RawFd,
    write_fd: RawFd,
}

#[derive(Debug, thiserror::Error)]
#[error("pipe error {value}")]
pub struct PipeError {
    value: i32,
}

impl Pipe {
    fn try_new() -> Result<Self, PipeError> {
        let mut fds: MaybeUninit<[libc::c_int; 2]> = MaybeUninit::uninit();
        let err = unsafe { libc::pipe(fds.as_mut_ptr() as *mut libc::c_int) };
        if err != 0 {
            return Err(PipeError { value: err });
        }
        let fds = unsafe { fds.assume_init() };
        Ok(Self {
            read_fd: fds[0] as RawFd,
            write_fd: fds[1] as RawFd,
        })
    }
}

struct LevelPattern(Regex);

impl LevelPattern {
    fn new() -> Self {
        Self(Regex::new(r"[^|]+\|[^|]+\| ([TDIWE]):").unwrap())
    }

    fn parse_level_from_line(&self, line: &str) -> Option<Level> {
        self.0.captures(line).map(|captures| match &captures[1] {
            "T" => Level::Trace,
            "D" => Level::Debug,
            "I" => Level::Info,
            "W" => Level::Warn,
            "E" => Level::Error,
            _ => panic!("Regex capture captures too much"),
        })
    }
}
fn redirect_output_to_log(fd: RawFd) -> Result<thread::JoinHandle<()>, PipeError> {
    let pipe = Pipe::try_new()?;
    let join_handle = thread::spawn(move || {
        let reader = unsafe { File::from_raw_fd(pipe.read_fd) };
        let reader = BufReader::new(reader);

        let level_pattern = LevelPattern::new();
        let mut log_level = Level::Info;

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if let Some(new_level) = level_pattern.parse_level_from_line(&line) {
                        log_level = new_level;
                    }
                    log::log!(log_level, "{line}");
                }
                Err(e) => {
                    log::error!("Could not read line from file descriptor {fd}: {e}");
                }
            };
        }
    });
    unsafe {
        libc::dup2(pipe.write_fd, fd);
        libc::close(pipe.write_fd);
    }
    Ok(join_handle)
}

pub struct LogRedirect {
    stdout_fd: RawFd,
    stderr_fd: RawFd,
    stdout_handle: thread::JoinHandle<()>,
    stderr_handle: thread::JoinHandle<()>,
}

impl LogRedirect {
    pub fn stop_and_wait(self) -> thread::Result<()> {
        unsafe {
            libc::close(self.stdout_fd);
            libc::close(self.stderr_fd);
        }
        self.stdout_handle.join()?;
        self.stderr_handle.join()?;
        Ok(())
    }
}

fn redirect_stdout_stderr_to_log_with_fd(stdout: RawFd, stderr: RawFd) -> Result<LogRedirect, PipeError> {
    Ok(LogRedirect {
        stdout_fd: stdout,
        stderr_fd: stderr,
        stdout_handle: redirect_output_to_log(stdout)?,
        stderr_handle: redirect_output_to_log(stderr)?,
    })
}

/// Capture all writing to stdout and stderr via a Unix pipe (libc::pipe)
/// and read it in a separate thread and log it via the logging library.
///
/// The logging library should be set to write to a Unix named pipe (fifo)
/// which can be read out by a separate sidecar container to stdout.
///
/// The LogRedirect can be used to stop and wait the processing threads.
pub fn redirect_stdout_stderr_to_log() -> Result<LogRedirect, PipeError> {
    redirect_stdout_stderr_to_log_with_fd(libc::STDOUT_FILENO, libc::STDERR_FILENO)
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;
    use std::io::Write;
    use std::os::fd::AsRawFd;

    use log::Level;
    use tempfile::tempfile;
    use tracing_test::traced_test;

    use super::LevelPattern;
    use super::redirect_stdout_stderr_to_log_with_fd;

    #[test]
    fn test_parse_log_level() {
        let level_pattern = LevelPattern::new();
        assert_eq!(
            level_pattern
                .parse_level_from_line("18.07.2025 18:07:20.250 | [00000001:00000001] C_Test | I: Info information"),
            Some(Level::Info)
        );
        assert_eq!(level_pattern.parse_level_from_line("Something else"), None);
    }

    #[traced_test]
    #[test]
    fn test_redirect_stdout_stderr_to_log_with_fd() {
        let stdout = tempfile().unwrap();
        let stderr = tempfile().unwrap();

        let log_redirect = redirect_stdout_stderr_to_log_with_fd(stdout.as_raw_fd(), stderr.as_raw_fd()).unwrap();

        writeln!(
            BufWriter::new(stdout),
            "18.07.2025 18:07:20.251 | [00000001:00000001] C_Test | W: Warning information"
        )
        .unwrap();
        writeln!(
            BufWriter::new(stderr),
            "18.07.2025 18:07:20.252 | [00000001:00000001] C_Test | E: Error information"
        )
        .unwrap();

        log_redirect.stop_and_wait().unwrap();

        assert!(!logs_contain(
            "18.07.2025 18:07:20.251 | [00000001:00000001] C_Test | W: Warning information"
        ));
        assert!(!logs_contain(
            "18.07.2025 18:07:20.252 | [00000001:00000001] C_Test | E: Error information"
        ));
    }
}
