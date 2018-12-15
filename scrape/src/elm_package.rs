//! A module for downloading & finding repository urls for packages
//! from [package.elm-lang.org](https://package.elm-lang.org).
//!
//! For example, if we wanted to iterate over all elm library url's we could
//! do something like this:
//!
//! ```
//! get_elm_libs()?
//!     .into_iter()
//!     .map(|r| find_git_url(&r))
//!     .for_each(|url| {
//!         // do something with each url
//!     });
//! ```
//!

use crate::git_repo::{GitRepo, GitError};
use crate::repo_cache::RepoCacheOptions;
use crate::chromium_dl::{chrome_dl, ChromeError};
use serde_derive::Deserialize;
use std::{fmt, error::Error};
use serde::de::IgnoredAny;
use std::path::Path;
use select::document::Document;
use select::predicate::{Predicate, Class, Attr};

const PACKAGES_BASE_URL: &str = "https://package.elm-lang.org";
const PACKAGES_SEARCH_URL: &str = "https://package.elm-lang.org/search.json";

/// Get a list of elm packages from [package.elm-lang.org](https://package.elm-lang.org)
/// # Error
/// Returns an error if there is a network failure or the data received by
/// [package.elm-lang.org](https://package.elm-lang.org) was not in the expected format.
pub fn get_elm_libs() -> Result<ElmPackageList, Box<Error>> {
    Ok(serde_json::from_str::<ElmPackageList>(
        reqwest::get(PACKAGES_SEARCH_URL)?.text()?.as_str(),
    )?)
}

pub type ElmPackageList = Vec<ElmPackageMetadata>;

/// The data returned from [package.elm-lang.org](https://package.elm-lang.org)
#[derive(Deserialize, Debug, Clone)]
pub struct ElmPackageMetadata {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

impl ElmPackageMetadata {
    /// find the path to the git repo in the cache on the filesystem
    pub fn get_repo_path(&self, o: &RepoCacheOptions) -> Result<String, ElmPackageError> {
        Path::new(o.cache_path.as_str())
            .join(Path::new(self.name.as_str()))
            .to_str()
            .ok_or_else(|| ElmPackageError::InvalidRepoPath(self.name.clone()))
            .map(|url| String::from(url))
    }

    /// Find the git url for a [ElmPackageMetadataRaw](struct.ElmPackageMetadataRaw.html)
    pub fn find_git_repo(&self, o: &RepoCacheOptions) -> Result<GitRepo, ElmPackageError> {
        let url = format!("{}/packages/{}/latest/", PACKAGES_BASE_URL, self.name);
        let page_text = chrome_dl(url.as_str(), o)?;
        let document = Document::from(page_text.as_str());
        for n in document.find(Class("pkg-nav-module").and(Attr("href", ()))) {
            if n.text().as_str() == "Browse Source" {
                if let Some(l) = n.attr("href") {
                    return Ok(GitRepo::from_url(l)?);
                }
            }
        }
        Err(ElmPackageError::CantFindUrl(url.clone()))
    }
}

#[derive(Debug)]
pub enum ElmPackageError {
    GitError(GitError),
    ChromeError(ChromeError),
    InvalidRepoPath(String),
    CantFindUrl(String),
}

impl Error for ElmPackageError {}

impl fmt::Display for ElmPackageError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match self {
            ElmPackageError::GitError(e) => write!(f, "{}", e),
            ElmPackageError::ChromeError(e) => write!(f, "{}", e),
            ElmPackageError::InvalidRepoPath(p) => write!(f, "invalid repository path: {}", p),
            ElmPackageError::CantFindUrl(u) => write!(f, "can't find url: {}", u),
        }
    }
}

impl From<GitError> for ElmPackageError {
    fn from(e: GitError) -> Self {
        ElmPackageError::GitError(e)
    }
}

impl From<ChromeError> for ElmPackageError {
    fn from(e: ChromeError) -> Self {
        ElmPackageError::ChromeError(e)
    }
}
