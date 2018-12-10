//! A module for caching or updating git repositories.

use crate::elm_package::{find_git_repo, ElmPackageMetadataRaw, FindGitUrlError, GitRepo};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process::Command;

/// Configuration options for caching the repositories.
pub struct RepoCacheOptions {
    /// root path for cache
    pub cache_path: String,
}

// Error indicating there was an error while git was running
#[derive(Debug)]
pub enum GitError {
    FindGitUrlError(FindGitUrlError),
    InvalidRepoPath(String),
    ExecuteError(std::io::Error),
    NonZeroExitCode(Option<i32>),
}

impl From<FindGitUrlError> for GitError {
    fn from(e: FindGitUrlError) -> Self {
        GitError::FindGitUrlError(e)
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        GitError::ExecuteError(e)
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitError::InvalidRepoPath(r) => write!(f, "invalid path for repo {}", r),
            GitError::FindGitUrlError(e) => write!(f, "{}", e),
            GitError::ExecuteError(e) => write!(f, "error while executing git: {}", e),
            GitError::NonZeroExitCode(c) => write!(f, "git returned non-zero exit code: {:?}", c),
        }
    }
}

impl Error for GitError {}

// find the path to the git repo in the cache on the filesystem
fn get_repo_path(m: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> Result<String, GitError> {
    Path::new(o.cache_path.as_str())
        .join(Path::new(m.name.as_str()))
        .to_str()
        .ok_or_else(|| GitError::InvalidRepoPath(m.name.clone()))
        .map(|url| String::from(url))
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
pub fn sync_repo(m: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> Result<(), GitError> {
    let repo_path = get_repo_path(m, o)?;
    let git_repo = match find_git_repo(m) {
        Ok(u) => u,
        Err(e) => {
            return Err(e.into());
        }
    };
    if Path::new(repo_path.as_str()).exists() {
        update_repo(&git_repo, repo_path.as_str())?;
    } else {
        clone_repo(&git_repo, repo_path.as_str())?;
    }
    Ok(())
}

fn clone_repo(git_repo: &GitRepo, repo_path: &str) -> Result<(), GitError> {
    let res = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(&[
            "clone",
            "--branch",
            git_repo.version.as_str(),
            "--depth",
            "1",
            git_repo.url.as_str(),
            repo_path,
        ])
        .output()?;
    if !res.status.success() {
        return Err(GitError::NonZeroExitCode(res.status.code()));
    }
    Ok(())
}

fn update_repo(git_repo: &GitRepo, repo_path: &str) -> Result<(), GitError> {
    Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(&["-C", repo_path, "pull", "--depth", "1", "--tags"])
        .output()?;
    Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(&["-C", repo_path, "checkout", git_repo.version.as_str()])
        .output()?;
    Ok(())
}
