use chrono::Utc;
use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::Duration as StdDuration;
use std::{error::Error, fmt, io};
use time::{Duration, OutOfRangeError};
use wait_timeout::ChildExt;

pub fn exec(cmd: &mut Command, timeout: StdDuration) -> Result<ExecResult, ExecError> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let mut stdout = Vec::with_capacity(4096);
    let mut stderr = Vec::with_capacity(4096);
    let mut buff = [0_u8; 4096];
    let mut stdout_done = false;
    let mut stderr_done = false;
    let start_time = Utc::now();
    let max_duration = Duration::from_std(timeout)?;
    loop {
        let now = Utc::now();
        let diff: Duration = now - start_time;
        if diff >= max_duration {
            let _ = child.kill();
            return Err(ExecError::TimeoutError { stdout, stderr });
        }
        let mut data_read = false;
        if !stdout_done {
            if let Some(o) = &mut child.stdout {
                let n = o.read(&mut buff)?;
                stdout.write(&buff[0..n])?;
                if n == 0 {
                    stdout_done = true;
                } else {
                    data_read = true;
                }
            }
        }
        if !stderr_done {
            if let Some(o) = &mut child.stderr {
                let n = o.read(&mut buff)?;
                stderr.write(&buff[0..n])?;
                if n == 0 {
                    stderr_done = true;
                } else {
                    data_read = true;
                }
            }
        }
        if stderr_done && stdout_done {
            let wait_time = diff.to_std()?;
            let res = child.wait_timeout(wait_time)?;
            if let Some(status) = res {
                if !status.success() {
                    return Err(ExecError::ProcessError {
                        status,
                        stdout,
                        stderr,
                    });
                }
            } else {
                return Err(ExecError::ProcessNotStarted);
            }
            return Ok(ExecResult { stdout, stderr });
        }
        if !data_read {
            sleep(StdDuration::from_millis(100));
        }
    }
}

pub struct ExecResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub enum ExecError {
    TimeoutError {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    IoError(io::Error),
    ProcessError {
        status: ExitStatus,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    ProcessNotStarted,
    DurationError(OutOfRangeError),
}

impl Error for ExecError {}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ExecError::TimeoutError { stdout, stderr } => write!(
                f,
                "TimeoutError:\nstdout:\n{}\nstderr:\n{}\n",
                String::from_utf8_lossy(stdout.as_slice()),
                String::from_utf8_lossy(stderr.as_slice())
            ),
            ExecError::ProcessError {
                status,
                stdout,
                stderr,
            } => write!(
                f,
                "ProcessError:\nstatus code: {:?}\nstdout:\n{}\nstderr:\n{}\n",
                status,
                String::from_utf8_lossy(stdout.as_slice()),
                String::from_utf8_lossy(stderr.as_slice())
            ),
            _ => write!(f, "{}", self),
        }
    }
}

impl From<io::Error> for ExecError {
    fn from(e: io::Error) -> Self {
        ExecError::IoError(e)
    }
}

impl From<OutOfRangeError> for ExecError {
    fn from(e: OutOfRangeError) -> Self {
        ExecError::DurationError(e)
    }
}
