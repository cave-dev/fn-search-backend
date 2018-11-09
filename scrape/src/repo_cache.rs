//! A module for caching or updating git repositories.

use elm_package::{ElmPackageMetadataRaw, find_git_url};
use std::error::Error;
use std::process::{Command, Output};
use std::path::Path;
use std::fmt;
use std::marker::Send;

pub type GitUrl = String;

/// Configuration options for caching the repositories.
pub struct RepoCacheOptions {
    /// root path for cache
    pub cache_path: String,
}

#[derive(Debug)]
struct InvalidRepoError {
    details: String,
}

impl InvalidRepoError {
    fn new(desc: String) -> Self {
        InvalidRepoError{
            details: desc,
        }
    }

    fn build(desc: String) -> Box<Self> {
        Box::new(InvalidRepoError::new(desc))
    }
}

impl fmt::Display for InvalidRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for InvalidRepoError {
    fn description(&self) -> &str {
        self.details.as_str()
    }
}

#[derive(Debug)]
struct GitError {
    details: String,
}

impl GitError {
    fn new(desc: String) -> Self {
        GitError{
            details: desc,
        }
    }

    fn build(desc: String) -> Box<Self> {
        Box::new(GitError::new(desc))
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for GitError {
    fn description(&self) -> &str {
        self.details.as_str()
    }
}

type BoxedError<T> = Result<T, Box<Error + Send>>;

fn get_repo_path(m: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> BoxedError<String> {
    Path::new(o.cache_path.as_str())
        .join(Path::new(m.name.as_str()))
        .to_str()
        .ok_or_else(|| -> Box<Error + Send> {
            InvalidRepoError::build(format!("invalid path for repo {}", m.name))
        })
        .map(|s| {
            String::from(s)
        })
}

/// Download or update all packages in an [ElmPackageMetadataRaw](../elm_package/struct.ElmPackageMetadataRaw.html)
/// # Errors
/// An error is returned on network error or git error
pub fn sync_repo(m: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> BoxedError<Output> {
    let repo_path = get_repo_path(m, o)?;
    let res = if Path::new(repo_path.as_str()).exists() {
        update_repo(repo_path.as_str())
    } else {
        clone_repo(&find_git_url(m), repo_path.as_str())
    }?;
    if !res.status.success() {
        Err(GitError::build(res.status.to_string()))
    } else {
        Ok(res)
    }
}

fn clone_repo(git_url: &GitUrl, repo_path: &str) -> BoxedError<Output> {
    Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(&["clone", "--depth", "1", git_url, repo_path])
        .output()
        .map_err(|e| -> Box<Error + Send> {
            Box::new(e)
        })
}

fn update_repo(repo_path: &str) -> BoxedError<Output> {
    Command::new("git")
        .args(&["-C", repo_path, "pull", "--depth", "1"])
        .output()
        .map_err(|e| -> Box<Error + Send> {
            Box::new(e)
        })
}
