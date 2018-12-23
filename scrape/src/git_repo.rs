use crate::repo_cache::RepoCacheOptions;
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use std::{error::Error, fmt, io};

pub struct GitRepo {
    pub url: String,
    pub version: String,
}

impl GitRepo {
    pub fn from_url(url: &str) -> Result<Self, GitError> {
        lazy_static! {
            static ref CLEANUP_REGEX: Regex =
                Regex::new(r"^(?P<url>http.+)/tree/(?P<version>[\.\d]+)$")
                    .expect("failed to build cleanup regex");
        }
        if let Some(captures) = CLEANUP_REGEX.captures(url) {
            if let Some(url) = captures.name("url") {
                if let Some(version) = captures.name("version") {
                    return Ok(GitRepo {
                        url: url.as_str().to_string(),
                        version: version.as_str().to_string(),
                    });
                }
            }
        }
        Err(GitError::ParseError(url.to_string()))
    }

    pub fn clone_repo(&self, repo_path: &str, o: &RepoCacheOptions) -> Result<(), GitError> {
        let res = Command::new(o.git_bin_path.as_str())
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(&[
                "clone",
                "--branch",
                self.version.as_str(),
                "--depth",
                "1",
                self.url.as_str(),
                repo_path,
            ])
            .output()?;
        if !res.status.success() {
            return match (String::from_utf8(res.stdout), String::from_utf8(res.stderr)) {
                (Ok(o), Ok(e)) => Err(GitError::ProcessError(res.status.code(), o, e)),
                _ => Err(GitError::ProcessErrorInvalidUtf8(res.status.code())),
            };
        }
        Ok(())
    }

    pub fn update_repo(&self, repo_path: &str, o: &RepoCacheOptions) -> Result<(), GitError> {
        let res = Command::new(o.git_bin_path.as_str())
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(&["-C", repo_path, "pull", "--depth", "1", "--tags"])
            .output()?;
        if !res.status.success() {
            return match (String::from_utf8(res.stdout), String::from_utf8(res.stderr)) {
                (Ok(o), Ok(e)) => Err(GitError::ProcessError(res.status.code(), o, e)),
                _ => Err(GitError::ProcessErrorInvalidUtf8(res.status.code())),
            };
        }
        let res = Command::new(o.git_bin_path.as_str())
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(&["-C", repo_path, "checkout", self.version.as_str()])
            .output()?;
        if !res.status.success() {
            return match (String::from_utf8(res.stdout), String::from_utf8(res.stderr)) {
                (Ok(o), Ok(e)) => Err(GitError::ProcessError(res.status.code(), o, e)),
                _ => Err(GitError::ProcessErrorInvalidUtf8(res.status.code())),
            };
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum GitError {
    ParseError(String),
    IoError(io::Error),
    ProcessError(Option<i32>, String, String),
    ProcessErrorInvalidUtf8(Option<i32>),
}

impl Error for GitError {}

impl fmt::Display for GitError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            GitError::ParseError(s) => write!(f, "error parsing git url {}", s),
            GitError::IoError(e) => write!(f, "io error while running git: {}", e),
            GitError::ProcessError(r, o, e) => write!(
                f,
                "git returned a non-zero status code: {:?}\nstdout:\n{}\nstderr:\n{}",
                r, o, e
            ),
            GitError::ProcessErrorInvalidUtf8(r) => write!(
                f,
                "git returned a non-zero status code and had invalid utf8 in stderr: {:?}",
                r
            ),
        }
    }
}

impl From<io::Error> for GitError {
    fn from(e: io::Error) -> Self {
        GitError::IoError(e)
    }
}
