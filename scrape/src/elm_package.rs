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

use crate::repo_cache::RepoCacheOptions;
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Predicate};
use serde::de::IgnoredAny;
use serde_json::from_str;
use std::error::Error as StdError;
use std::fmt;
use std::process::Command;

const PACKAGES_BASE_URL: &str = "https://package.elm-lang.org";
const PACKAGES_SEARCH_URL: &str = "https://package.elm-lang.org/search.json";

/// Get a list of elm packages from [package.elm-lang.org](https://package.elm-lang.org)
/// # Error
/// Returns an error if there is a network failure or the data received by
/// [package.elm-lang.org](https://package.elm-lang.org) was not in the expected format.
pub fn get_elm_libs() -> Result<ElmPackageMetadataListRaw, Box<StdError>> {
    Ok(from_str::<ElmPackageMetadataListRaw>(
        reqwest::get(PACKAGES_SEARCH_URL)?.text()?.as_str(),
    )?)
}

/// The data returned from [package.elm-lang.org](https://package.elm-lang.org)
#[derive(Deserialize, Debug, Clone)]
pub struct ElmPackageMetadataRaw {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

type ElmPackageMetadataListRaw = Vec<ElmPackageMetadataRaw>;

#[derive(Debug)]
pub enum Error {
    CantFindUrl(String),
    RequestError(reqwest::Error),
    CliError(std::io::Error),
    ChromiumError(Option<i32>),
    PageParseError(std::string::FromUtf8Error),
    ParseUrlAndVersionError(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::RequestError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::CliError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::PageParseError(e)
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match self {
            Error::CantFindUrl(s) => write!(f, "can't find url for {}", s),
            Error::RequestError(e) => write!(f, "{}", e),
            Error::CliError(e) => write!(f, "error while running chromium CLI: {}", e),
            Error::ChromiumError(c) => write!(f, "chrome returned non-zero exit code: {:?}", c),
            Error::PageParseError(e) => write!(f, "package page returned invalid utf8: {}", e),
            Error::ParseUrlAndVersionError(u) => {
                write!(f, "error getting url and version from: {}", u)
            }
        }
    }
}

fn chrome_dl(url: &str, o: &RepoCacheOptions) -> Result<String, Error> {
    let output = Command::new(o.chromium_bin_path.as_str())
        .args(&["--headless", "--disable-gpu", "--dump-dom", url])
        .output()?;
    if !output.status.success() {
        return Err(Error::ChromiumError(output.status.code()));
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub struct GitRepo {
    pub url: String,
    pub version: String,
}

fn cleanup_url(url: &str) -> Result<GitRepo, Error> {
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
    Err(Error::ParseUrlAndVersionError(url.to_string()))
}

/// Find the git url for a [ElmPackageMetadataRaw](struct.ElmPackageMetadataRaw.html)
pub fn find_git_repo(ur: &ElmPackageMetadataRaw, o: &RepoCacheOptions) -> Result<GitRepo, Error> {
    let url = format!("{}/packages/{}/latest/", PACKAGES_BASE_URL, ur.name);
    let page_text = chrome_dl(url.as_str(), o)?;
    let document = Document::from(page_text.as_str());
    for n in document.find(Class("pkg-nav-module").and(Attr("href", ()))) {
        if n.text().as_str() == "Browse Source" {
            if let Some(l) = n.attr("href") {
                return cleanup_url(l);
            }
        }
    }
    Err(Error::CantFindUrl(url.clone()))
}
