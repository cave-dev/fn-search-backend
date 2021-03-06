//! A module for caching or updating git repositories.

use crate::db_queries::{update_repo, UpdateUrlError};
use crate::elm_package::{ElmPackage, ElmPackageError};
use crate::git_repo::GitError;
use fn_search_backend::Config;
use std::path::Path;
use std::{error::Error, fmt};

/// Configuration options for caching the repositories.
pub struct RepoCacheOptions {
    /// root path for cache
    pub cache_path: String,
    pub chromium_bin_path: String,
    pub git_bin_path: String,
}

pub enum SyncResult {
    Update,
    Clone,
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
pub fn sync_repo(
    m: &ElmPackage,
    o: &RepoCacheOptions,
    config: &Config,
) -> Result<SyncResult, SyncRepoError> {
    let repo_path = m.get_repo_path(o)?;
    let git_repo = m.find_git_repo(config, o)?;
    update_repo(
        &config.db,
        m.name.as_str(),
        git_repo.url.as_str(),
        git_repo.version.as_str(),
    )?;
    if Path::new(repo_path.as_str()).exists() {
        git_repo.update_repo(repo_path.as_str(), config, o)?;
        Ok(SyncResult::Update)
    } else {
        git_repo.clone_repo(repo_path.as_str(), config, o)?;
        Ok(SyncResult::Clone)
    }
}

#[derive(Debug)]
pub enum SyncRepoError {
    GitError(GitError),
    ElmPackageError(ElmPackageError),
    UpdateUrlError(UpdateUrlError),
}

impl Error for SyncRepoError {}

impl fmt::Display for SyncRepoError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            SyncRepoError::GitError(e) => write!(f, "{}", e),
            SyncRepoError::ElmPackageError(e) => write!(f, "{}", e),
            SyncRepoError::UpdateUrlError(e) => write!(f, "{}", e),
        }
    }
}

impl From<GitError> for SyncRepoError {
    fn from(e: GitError) -> Self {
        SyncRepoError::GitError(e)
    }
}

impl From<ElmPackageError> for SyncRepoError {
    fn from(e: ElmPackageError) -> Self {
        SyncRepoError::ElmPackageError(e)
    }
}

impl From<UpdateUrlError> for SyncRepoError {
    fn from(e: UpdateUrlError) -> Self {
        SyncRepoError::UpdateUrlError(e)
    }
}
