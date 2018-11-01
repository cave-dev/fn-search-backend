
use elm_package::{ElmPackageMetadata, find_git_url};
use std::error::Error;
use std::process::{Command, Output};
use std::path::Path;
use std::fmt;
use std::marker::Send;

pub type GitUrl = String;

pub struct RepoCacheOptions {
    // root path for cache
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

type BoxedError<T> = Result<T, Box<Error + Send>>;

fn get_repo_path(m: &ElmPackageMetadata, o: &RepoCacheOptions) -> BoxedError<String> {
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

pub fn sync_repo(m: &ElmPackageMetadata, o: &RepoCacheOptions) -> BoxedError<Output> {
    let repo_path = get_repo_path(m, o)?;
    if Path::new(repo_path.as_str()).exists() {
        update_repo(repo_path.as_str())
    } else {
        clone_repo(&find_git_url(m), repo_path.as_str())
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
