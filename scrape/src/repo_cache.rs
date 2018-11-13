//! A module for caching or updating git repositories.

use crate::elm_package::GitUrl;
use crate::elm_package::{find_git_url, ElmPackageMetadataRaw};
use std::error::Error;
use std::fmt;
use std::marker::Send;
use std::path::Path;
use std::process::{Command, Output};

/// Configuration options for caching the repositories.
pub struct RepoCacheOptions {
    /// root path for cache
    pub cache_path: String,
}

// Error indicating the repository is invalid
#[derive(Debug)]
struct InvalidRepoError {
    details: String,
}

impl InvalidRepoError {
    fn new(desc: String) -> Self {
        InvalidRepoError { details: desc }
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

// Error indicating there was an error while git was running
#[derive(Debug)]
struct GitError {
    details: String,
}

impl GitError {
    fn new(desc: String) -> Self {
        GitError { details: desc }
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

// find the path to the git repo in the cache on the filesystem
fn get_repo_path(m: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> BoxedError<String> {
    Path::new(o.cache_path.as_str())
        .join(Path::new(m.name.as_str()))
        .to_str()
        .ok_or_else(|| -> Box<Error + Send> {
            InvalidRepoError::build(format!("invalid path for repo {}", m.name))
        })
        .map(|s| String::from(s))
}

/// Download or update all packages in an [ElmPackageMetadataRaw](../elm_package/struct.ElmPackageMetadataRaw.html)
/// # Errors
/// An error is returned on network error or git error
/// # Example
/// ```ignore
/// use fn_search_backend_scrape::elm_package::get_elm_libs;
///
/// let options = RepoCacheOptions{cache_path: String::from("/path/to/cache")}
/// let res = get_elm_libs()?
///     .into_iter()
///     .map(|pkg| {
///         sync_repo(&pkg, &options)
///     });
/// // Potentially do something with the results/errors
/// ```
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
        .map_err(|e| -> Box<Error + Send> { Box::new(e) })
}

fn update_repo(repo_path: &str) -> BoxedError<Output> {
    Command::new("git")
        .args(&["-C", repo_path, "pull", "--depth", "1"])
        .output()
        .map_err(|e| -> Box<Error + Send> { Box::new(e) })
}
