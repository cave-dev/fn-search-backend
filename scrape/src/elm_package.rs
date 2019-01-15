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

use crate::chromium_dl::{chrome_dl, ChromeError};
use crate::git_repo::{GitError, GitRepo};
use crate::repo_cache::RepoCacheOptions;
use fn_search_backend_parsers::{get_elm_exports, ElmExports};
use glob::{glob, GlobError, PatternError};
use select::document::Document;
use select::predicate::{Attr, Class, Predicate};
use serde::de::IgnoredAny;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::{error::Error, fmt};

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

pub type ElmPackageList = Vec<ElmPackage>;

/// The data returned from [package.elm-lang.org](https://package.elm-lang.org)
#[derive(Deserialize, Debug, Clone)]
pub struct ElmPackage {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

impl ElmPackage {
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

    // get the exports of a elm package
    // returns an error, or a vector of results which are either ElmFiles, or the path to a file that failed to parse
    pub fn get_exports(
        &self,
        o: &RepoCacheOptions,
    ) -> Result<Vec<Result<ElmFile, String>>, ElmPackageError> {
        let mut exports = vec![];
        let path = self.get_repo_path(&o)?;
        for res in glob(format!("{}/src/**/*.elm", path).as_str())? {
            let res = res?;
            let path = res.as_path();
            let mut file = File::open(path)?;
            let mut elm_code = String::new();
            file.read_to_string(&mut elm_code)?;
            match get_elm_exports(elm_code.as_str()) {
                Ok(e) => exports.push(Ok(ElmFile {
                    repository: path
                        .to_str()
                        .expect("cache path was not convertible into a string")
                        .to_string(),
                    path: path.to_str().unwrap_or_default().to_string(),
                    exports: e,
                })),
                Err(_) => exports.push(Err(path.to_str().unwrap_or_default().to_string())),
            };
        }
        Ok(exports)
    }
}

#[derive(Debug, Clone)]
pub struct ElmFile {
    pub repository: String,
    pub path: String,
    pub exports: ElmExports,
}

#[derive(Debug)]
pub enum ElmPackageError {
    GitError(GitError),
    ChromeError(ChromeError),
    InvalidRepoPath(String),
    CantFindUrl(String),
    GlobError(GlobError),
    GlobPatternError(PatternError),
    IoError(io::Error),
}

impl Error for ElmPackageError {}

impl fmt::Display for ElmPackageError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match self {
            ElmPackageError::GitError(e) => write!(f, "{}", e),
            ElmPackageError::ChromeError(e) => write!(f, "{}", e),
            ElmPackageError::InvalidRepoPath(p) => write!(f, "invalid repository path: {}", p),
            ElmPackageError::CantFindUrl(u) => write!(f, "can't find url: {}", u),
            ElmPackageError::GlobError(e) => write!(f, "error while globbing: {}", e),
            ElmPackageError::GlobPatternError(e) => write!(f, "invalid glob pattern: {}", e),
            ElmPackageError::IoError(e) => write!(f, "io error while getting exports: {}", e),
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

impl From<GlobError> for ElmPackageError {
    fn from(e: GlobError) -> Self {
        ElmPackageError::GlobError(e)
    }
}

impl From<PatternError> for ElmPackageError {
    fn from(e: PatternError) -> Self {
        ElmPackageError::GlobPatternError(e)
    }
}

impl From<io::Error> for ElmPackageError {
    fn from(e: io::Error) -> Self {
        ElmPackageError::IoError(e)
    }
}
