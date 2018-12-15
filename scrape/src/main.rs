//! # fn-search-backend-cache
//!
//! Caching the functions found on [packages.elm-lang.org](https://packages.elm-lang.org)
//! is performed with the following algorithm
//!
//! * Download the list of packages on [packages.elm-lang.org](https://packages.elm-lang.org)
//! * Iterate over each repository in parallel
//!   * Check if the repository already is cached
//!     * If yes, spawn a subprocess and run git pull to update the repository
//!     * If no, spawn a subprocess and run git clone to download the repository
//!   * Run a Elm parser on the source code to find all exported functions/variables/etc...
//!   * Insert exported functions and types into the database

pub mod elm_package;
pub mod repo_cache;
pub mod repo_parse;
pub mod chromium_dl;
pub mod git_repo;

use crate::elm_package::{ElmPackageMetadata, ElmPackageError};
use crate::repo_cache::{sync_repo, RepoCacheOptions, SyncRepoError, SyncResult};
use clap::{clap_app, crate_version, crate_authors, crate_description};
use rayon::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let matches = clap_app!(myapp =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CACHE_DIR: -d --("cache-dir") +takes_value +required "directory for repositories to be cached in")
        (@arg CHROME: -c --chrome +takes_value +required default_value("chromium") "google chrome or chromium executable")
        (@arg GIT: -g --git +takes_value +required default_value("git") "git executable")
    ).get_matches();

    let cfg_file = matches
        .value_of("CACHE_DIR")
        .expect("error, no cache directory specified");
    let chrome = matches.value_of("CHROME").unwrap();
    let git = matches.value_of("GIT").unwrap();
    let config = RepoCacheOptions {
        cache_path: String::from(cfg_file),
        chromium_bin_path: chrome.to_string(),
        git_bin_path: git.to_string(),
    };
    let elm_libs = elm_package::get_elm_libs()?;
    let failed_libs: Vec<&ElmPackageMetadata> = elm_libs
        .par_iter()
        .map(|i| (i, sync_repo(i, &config)))
        .filter_map(|r| match r.1 {
            Ok(res) => {
                match res {
                    SyncResult::Clone => println!("cloned repo {}", r.0.name),
                    SyncResult::Update => println!("updated repo {}", r.0.name),
                }
                None
            }
            Err(e) => {
                if let SyncRepoError::ElmPackageError(ElmPackageError::CantFindUrl(_)) = &e {
                    // chrome doesn't finish downloading the page sometimes, try again
                    return Some(r.0);
                }
                eprintln!("error syncing repo {}: {}", r.0.name, e);
                None
            }
        })
        .collect();
    // try again for failed libs
    failed_libs
        .par_iter()
        .map(|i| (i, sync_repo(i, &config)))
        .for_each(|r| match r.1 {
            Ok(_) => {
                println!("cloned or updated repo {}", r.0.name);
            }
            Err(e) => {
                eprintln!("error syncing repo {}: {}", r.0.name, e);
            }
        });
    Ok(())
}
