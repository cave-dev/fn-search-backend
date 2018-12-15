
use std::process::Command;
use std::{fmt, io, error::Error};
use std::string::FromUtf8Error;
use crate::repo_cache::RepoCacheOptions;

pub fn chrome_dl(url: &str, o: &RepoCacheOptions) -> Result<String, ChromeError> {
    let res = Command::new(o.chromium_bin_path.as_str())
        .args(&["--headless", "--disable-gpu", "--dump-dom", url])
        .output()?;
    if !res.status.success() {
        return match String::from_utf8(res.stderr) {
            Ok(e) => Err(ChromeError::ProcessError(res.status.code(), e)),
            Err(_) => Err(ChromeError::ProcessErrorInvalidUtf8(res.status.code())),
        };
    }
    Ok(String::from_utf8(res.stdout)?)
}

#[derive(Debug)]
pub enum ChromeError {
    ProcessError(Option<i32>, String),
    ProcessErrorInvalidUtf8(Option<i32>),
    ParseError(FromUtf8Error),
    IoError(io::Error),
}

impl Error for ChromeError {}

impl fmt::Display for ChromeError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            ChromeError::ProcessError(r, e) => write!(f, "chromium returned a non-zero status code: {:?}\n{}", r, e),
            ChromeError::ProcessErrorInvalidUtf8(r) => write!(f, "chromium returned a non-zero status code and had invalid utf8 in stderr: {:?}", r),
            ChromeError::ParseError(p) => write!(f, "error parsing page from chromium: {}", p),
            ChromeError::IoError(e) => write!(f, "io error while executing chromium: {}", e),
        }
    }
}

impl From<FromUtf8Error> for ChromeError {
    fn from(e: FromUtf8Error) -> Self {
        ChromeError::ParseError(e)
    }
}

impl From<io::Error> for ChromeError {
    fn from(e: io::Error) -> Self {
        ChromeError::IoError(e)
    }
}
