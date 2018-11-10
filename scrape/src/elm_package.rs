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

use std::error::Error;
use serde_json::from_str;
use serde::de::IgnoredAny;

/// URL to a git repository
pub type GitUrl = String;

const PACKAGES_SEARCH_URL: &'static str = "https://package.elm-lang.org/search.json";

/// Get a list of elm packages from [package.elm-lang.org](https://package.elm-lang.org)
/// # Error
/// Returns an error if there is a network failure or the data received by
/// [package.elm-lang.org](https://package.elm-lang.org) was not in the expected format.
pub fn get_elm_libs() -> Result<ElmPackageMetadataListRaw, Box<Error>> {
    Ok(from_str::<ElmPackageMetadataListRaw>(
        reqwest::get(PACKAGES_SEARCH_URL)?
            .text()?
            .as_str()
    )?)
}

/// The data returned from [package.elm-lang.org](https://package.elm-lang.org)
#[derive(Deserialize, Debug)]
pub struct ElmPackageMetadataRaw {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

type ElmPackageMetadataListRaw = Vec<ElmPackageMetadataRaw>;

/// Find the git url for a [ElmPackageMetadataRaw](struct.ElmPackageMetadataRaw.html)
pub fn find_git_url(ur: &ElmPackageMetadataRaw) -> GitUrl {
    //TODO add logic to find it, for now assume github
    format!("https://github.com/{}", ur.name)
}
