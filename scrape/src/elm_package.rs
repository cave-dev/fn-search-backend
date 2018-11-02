
use std::error::Error;
use serde_json::from_str;
use serde::de::IgnoredAny;
use repo_cache::GitUrl;
use fn_search_backend_db::models::NewRepository;

const PACKAGES_SEARCH_URL: &'static str = "https://package.elm-lang.org/search.json";

pub fn get_elm_libs() -> Result<ElmPackageMetadataListRaw, Box<Error>> {
    Ok(from_str::<ElmPackageMetadataListRaw>(
        reqwest::get(PACKAGES_SEARCH_URL)?
            .text()?
            .as_str()
    )?)
}

#[derive(Deserialize, Debug)]
pub struct ElmPackageMetadataRaw {
    pub name: String,
    summary: IgnoredAny,
    license: IgnoredAny,
    versions: IgnoredAny,
}

type ElmPackageMetadataListRaw = Vec<ElmPackageMetadataRaw>;

pub fn find_git_url(ur: &ElmPackageMetadataRaw) -> GitUrl {
    //TODO add logic to find it, for now assume github
    format!("https://github.com/{}", ur.name)
}
