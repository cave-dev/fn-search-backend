
use std::error::Error;
use serde_json::from_str;
use serde::de::IgnoredAny;
use repo_cache::GitUrl;

const PACKAGES_SEARCH_URL: &'static str = "https://package.elm-lang.org/search.json";

pub fn get_elm_libs() -> Result<ElmPackageMetadataList, Box<Error>> {
    Ok(from_str(
        reqwest::get(PACKAGES_SEARCH_URL)?
            .text()?
            .as_str()
    )?)
}

#[derive(Deserialize, Debug)]
pub struct ElmPackageMetadata {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

pub type ElmPackageMetadataList = Vec<ElmPackageMetadata>;


pub fn find_git_url(ur: &ElmPackageMetadata) -> GitUrl {
    //TODO add logic to find it, for now assume github
    format!("https://github.com/{}", ur.name)
}
