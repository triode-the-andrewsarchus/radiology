//! Bounded, cancellable subprocess execution for indexers and tool probes.
//!
//! This blocking runner belongs on the shared read-worker pool. It never drives
//! a GUI runtime, and a child owns a separate POSIX process group. Cancellation,
//! deadlines, I/O failures and unwinding kill the group and reap the direct child.

use super::command::IndexCommand;
use std::collections::VecDeque;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const CAPTURE_PART: usize = 32 * 1024;

/// Bounded first/last diagnostics, with the full byte count retained.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CapturedOutput {
    /// UTF-8 lossy diagnostic text, with an omission marker when truncated.
    pub text: String,
    /// Bytes drained from the child's stream, before truncation.
    pub total_bytes: u64,
    /// Whether middle bytes were omitted from the retained diagnostics.
    pub truncated: bool,
}

/// Result of a completed process, including nonzero exits.
#[derive(Debug)]
pub struct RunOutput {
    /// Numeric exit code, absent when the process was killed by a signal.
    pub exit_code: Option<i32>,
    /// POSIX terminating signal, if any.
    pub signal: Option<i32>,
    /// Bounded stdout diagnostics.
    pub stdout: CapturedOutput,
    /// Bounded stderr diagnostics.
    pub stderr: CapturedOutput,
    /// Time spent running the process.
    pub elapsed: Duration,
}

impl RunOutput {
    /// Returns whether the tool exited normally with status zero.
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0) && self.signal.is_none()
    }
}

/// Why a subprocess could not complete.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunFailureKind {
    /// The owner retired or explicitly cancelled the job.
    #[error("cancelled")]
    Cancelled,
    /// The specified wall-clock deadline expired.
    #[error("timed out")]
    TimedOut,
    /// Spawning, reading output or waiting failed.
    #[error("{0}")]
    Io(String),
}

/// Process failure with diagnostics preserved through cancellation and timeout.
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct RunFailure {
    /// Failure category.
    pub kind: RunFailureKind,
    /// Bounded stdout captured before failure.
    pub stdout: CapturedOutput,
    /// Bounded stderr captured before failure.
    pub stderr: CapturedOutput,
}

/// Runs a prepared command with bounded output, cancellation and a deadline.
///
/// Resource files must already have been materialized by the indexing workspace.
/// No output pipe reader threads or output log files are created. A continuously
/// writing child cannot starve cancellation because each drain has a byte budget.
///
/// # Errors
///
/// Returns [`RunFailure`] on cancellation, timeout, process or I/O failure.
/// POSIX process groups are required; other platforms return an explicit error.
pub fn run(
    command: &IndexCommand,
    cancel: &AtomicBool,
    timeout: Duration,
) -> Result<RunOutput, RunFailure> {
    run_observed(command, cancel, timeout, |_, _| Ok(()))
}

/// Identifies one drained subprocess output stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Runs a command while delivering complete byte chunks to a bounded consumer.
///
/// Captured diagnostics remain bounded independently of this observer. The
/// observer must return promptly and enforce its own domain limits. Returning an
/// I/O error aborts the child group; no truncated stream is reported as complete.
///
/// # Errors
///
/// Returns process failures from [`run`] or an observer's error.
pub fn run_observed(
    command: &IndexCommand,
    cancel: &AtomicBool,
    timeout: Duration,
    mut observer: impl FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> Result<RunOutput, RunFailure> {
    let mut stdout = Capture::default();
    let mut stderr = Capture::default();
    let result = run_inner(
        command,
        cancel,
        timeout,
        &mut stdout,
        &mut stderr,
        &mut observer,
    );
    let stdout = stdout.finish();
    let stderr = stderr.finish();
    match result {
        Ok((exit_code, signal, elapsed)) => Ok(RunOutput {
            exit_code,
            signal,
            stdout,
            stderr,
            elapsed,
        }),
        Err(kind) => Err(RunFailure {
            kind,
            stdout,
            stderr,
        }),
    }
}

#[derive(Default)]
struct Capture {
    first: Vec<u8>,
    last: VecDeque<u8>,
    total: u64,
}

impl Capture {
    fn push(&mut self, bytes: &[u8]) {
        self.total = self.total.saturating_add(bytes.len() as u64);
        let prefix = (CAPTURE_PART - self.first.len()).min(bytes.len());
        self.first.extend_from_slice(&bytes[..prefix]);
        let rest = &bytes[prefix..];
        if rest.len() >= CAPTURE_PART {
            self.last.clear();
            self.last.extend(&rest[rest.len() - CAPTURE_PART..]);
        } else {
            let overflow = (self.last.len() + rest.len()).saturating_sub(CAPTURE_PART);
            self.last.drain(..overflow);
            self.last.extend(rest);
        }
    }

    fn finish(self) -> CapturedOutput {
        let truncated = self.total > (CAPTURE_PART * 2) as u64;
        let mut text = String::from_utf8_lossy(&self.first).into_owned();
        if truncated {
            text.push_str("\n… output omitted …\n");
        }
        text.push_str(&String::from_utf8_lossy(
            &self.last.into_iter().collect::<Vec<_>>(),
        ));
        CapturedOutput {
            text,
            total_bytes: self.total,
            truncated,
        }
    }
}

#[cfg(unix)]
fn run_inner(
    command: &IndexCommand,
    cancel: &AtomicBool,
    timeout: Duration,
    out: &mut Capture,
    err: &mut Capture,
    observer: &mut dyn FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> Result<(Option<i32>, Option<i32>, Duration), RunFailureKind> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    if cancel.load(Ordering::Relaxed) {
        return Err(RunFailureKind::Cancelled);
    }
    if timeout.is_zero() {
        return Err(RunFailureKind::TimedOut);
    }
    let started = Instant::now();
    let child = Command::new(&command.executable)
        .args(&command.arguments)
        .current_dir(&command.directory)
        .envs(&command.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(io_failure)?;
    let mut group = ProcessGroup(child);
    let mut stdout = group.0.stdout.take().expect("piped stdout");
    let mut stderr = group.0.stderr.take().expect("piped stderr");
    nonblocking(&stdout).map_err(io_failure)?;
    nonblocking(&stderr).map_err(io_failure)?;
    loop {
        drain_streams(&mut stdout, &mut stderr, out, err, observer).map_err(io_failure)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(RunFailureKind::Cancelled);
        }
        if started.elapsed() >= timeout {
            return Err(RunFailureKind::TimedOut);
        }
        if let Some((code, signal)) = poll_exit(&group.0).map_err(io_failure)? {
            // Read bytes emitted between the preceding drain and process exit.
            drain_streams(&mut stdout, &mut stderr, out, err, observer).map_err(io_failure)?;
            return Ok((code, signal, started.elapsed()));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn io_failure(error: io::Error) -> RunFailureKind {
    RunFailureKind::Io(error.to_string())
}

#[cfg(unix)]
fn nonblocking(pipe: &impl std::os::fd::AsRawFd) -> io::Result<()> {
    let descriptor = pipe.as_raw_fd();
    // SAFETY: the owned pipe keeps this valid descriptor open for both fcntl calls.
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: F_SETFL changes flags on this owned descriptor and takes an integer.
    if unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(unix)]
fn poll_exit(child: &std::process::Child) -> io::Result<Option<(Option<i32>, Option<i32>)>> {
    // SAFETY: all-zero siginfo is a valid output buffer; waitid initializes it.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    // Keep the leader waitable until ProcessGroup drops. Its unreaped PID
    // reserves the group identity while cleanup signals descendants.
    // SAFETY: info points to a valid writable siginfo_t, and P_PID uses our child.
    if unsafe {
        libc::waitid(
            libc::P_PID,
            child.id(),
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    } < 0
    {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(None);
        }
        return Err(error);
    }
    // SAFETY: waitid with WEXITED initializes the SIGCHLD fields; zero si_pid
    // denotes no pending exit when WNOHANG is used.
    unsafe {
        if info.si_pid() == 0 {
            return Ok(None);
        }
        Ok(Some(if info.si_code == libc::CLD_EXITED {
            (Some(info.si_status()), None)
        } else {
            (None, Some(info.si_status()))
        }))
    }
}

#[cfg(unix)]
struct ProcessGroup(std::process::Child);

#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // SAFETY: process_group(0) assigned the child's positive PID as its own
        // group ID. Negating it targets that group, never the reader's group.
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        let _ = self.0.wait();
    }
}

#[cfg(unix)]
fn drain_streams(
    stdout: &mut std::process::ChildStdout,
    stderr: &mut std::process::ChildStderr,
    out: &mut Capture,
    err: &mut Capture,
    observer: &mut dyn FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let streams: [(&mut dyn Read, &mut Capture, OutputStream); 2] = [
        (stdout, out, OutputStream::Stdout),
        (stderr, err, OutputStream::Stderr),
    ];
    for (reader, capture, stream) in streams {
        drain(reader, capture, &mut |bytes| observer(stream, bytes))?;
    }
    Ok(())
}

#[cfg(unix)]
fn drain(
    reader: &mut (impl Read + ?Sized),
    capture: &mut Capture,
    observe: &mut impl FnMut(&[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let mut buffer = [0_u8; 8192];
    // Bound each turn even when a child continuously fills either stream.
    for _ in 0..32 {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => {
                capture.push(&buffer[..count]);
                observe(&buffer[..count])?;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn run_inner(
    _: &IndexCommand,
    _: &AtomicBool,
    _: Duration,
    _: &mut Capture,
    _: &mut Capture,
    _: &mut dyn FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> Result<(Option<i32>, Option<i32>, Duration), RunFailureKind> {
    Err(RunFailureKind::Io(
        "Indexer process groups require a POSIX host".into(),
    ))
}

#[cfg(all(test, unix))]
mod tests;
