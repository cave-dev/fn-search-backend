use crate::repo_cache::RepoCacheOptions;
use crate::subprocess::{exec, ExecError};
use std::process::Command;
use std::string::FromUtf8Error;
use std::time::Duration;
use std::{error::Error, fmt};

pub fn chrome_dl(url: &str, o: &RepoCacheOptions) -> Result<String, ChromeError> {
    let res = exec(
        &mut Command::new(o.chromium_bin_path.as_str()).args(&[
            "--headless",
            "--disable-gpu",
            "--dump-dom",
            url,
        ]),
        Duration::from_secs(10),
    )?;
    let stdout = res.stdout;
    match String::from_utf8(stdout) {
        Ok(output) => Ok(output),
        Err(e) => Err(ChromeError::StdoutParseError(e)),
    }
}

#[derive(Debug)]
pub enum ChromeError {
    ProcessError(ExecError),
    StdoutParseError(FromUtf8Error),
}

impl Error for ChromeError {}

impl fmt::Display for ChromeError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            ChromeError::ProcessError(e) => write!(f, "error during execution of chrome: {}", e),
            ChromeError::StdoutParseError(e) => {
                write!(f, "chrome returned invalid utf8 in output: {}", e)
            }
        }
    }
}

impl From<ExecError> for ChromeError {
    fn from(e: ExecError) -> Self {
        ChromeError::ProcessError(e)
    }
}
