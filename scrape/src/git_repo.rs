use crate::repo_cache::RepoCacheOptions;
use crate::subprocess::{exec, ExecError};
use fn_search_backend::Config;
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use std::time::Duration;
use std::{error::Error, fmt};

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

    pub fn clone_repo(
        &self,
        repo_path: &str,
        config: &Config,
        o: &RepoCacheOptions,
    ) -> Result<(), GitError> {
        exec(
            &mut Command::new(o.git_bin_path.as_str())
                .env("GIT_TERMINAL_PROMPT", "0")
                .args(&[
                    "clone",
                    "--branch",
                    self.version.as_str(),
                    "--depth",
                    "1",
                    self.url.as_str(),
                    repo_path,
                ]),
            Duration::from_secs(config.scrape.git_timeout),
        )?;
        Ok(())
    }

    pub fn update_repo(
        &self,
        repo_path: &str,
        config: &Config,
        o: &RepoCacheOptions,
    ) -> Result<(), GitError> {
        exec(
            &mut Command::new(o.git_bin_path.as_str())
                .env("GIT_TERMINAL_PROMPT", "0")
                .args(&["-C", repo_path, "fetch", "--depth", "1", "--tags", "origin"]),
            Duration::from_secs(config.scrape.git_timeout),
        )?;
        exec(
            &mut Command::new(o.git_bin_path.as_str())
                .env("GIT_TERMINAL_PROMPT", "0")
                .args(&["-C", repo_path, "reset", "--hard", self.version.as_str()]),
            Duration::from_secs(5),
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum GitError {
    ParseError(String),
    ProcessError(ExecError),
}

impl Error for GitError {}

impl fmt::Display for GitError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            GitError::ParseError(s) => write!(f, "error parsing git url {}", s),
            GitError::ProcessError(e) => write!(f, "error while running git: {}", e),
        }
    }
}

impl From<ExecError> for GitError {
    fn from(e: ExecError) -> Self {
        GitError::ProcessError(e)
    }
}
