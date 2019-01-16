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

pub mod chromium_dl;
pub mod db_queries;
pub mod elm_package;
pub mod git_repo;
pub mod repo_cache;

use crate::db_queries::insert_functions;
use crate::elm_package::{ElmFile, ElmPackage, ElmPackageError};
use crate::repo_cache::{sync_repo, RepoCacheOptions, SyncRepoError, SyncResult};
use clap::{clap_app, crate_authors, crate_description, crate_version, ArgMatches};
use fn_search_backend::{get_config, Config};
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;

fn sync(cfg: &Config, cache_config: &RepoCacheOptions) -> Result<(), Box<Error>> {
    let elm_libs = elm_package::get_elm_libs()?;
    let failed_libs: Vec<&ElmPackage> = elm_libs
        .par_iter()
        .map(|i| (i, sync_repo(i, &cache_config, &cfg.db)))
        .filter_map(|r| match r.1 {
            Ok(res) => {
                match res {
                    SyncResult::Clone => println!("cloned repo {}", r.0.name),
                    SyncResult::Update => println!("updated repo {}", r.0.name),
                }
                None
            }
            Err(e) => {
                match &e {
                    // chrome doesn't finish downloading the page sometimes, try again
                    SyncRepoError::ElmPackageError(ElmPackageError::CantFindUrl(_)) => Some(r.0),
                    _ => {
                        eprintln!("error syncing repo {}: {}", r.0.name, e);
                        None
                    }
                }
            }
        })
        .collect();
    // try failed libs again
    failed_libs
        .par_iter()
        .map(|i| (i, sync_repo(i, &cache_config, &cfg.db)))
        .for_each(|r| match r.1 {
            Ok(res) => {
                match res {
                    SyncResult::Clone => println!("cloned repo {}", r.0.name),
                    SyncResult::Update => println!("updated repo {}", r.0.name),
                };
            }
            Err(e) => {
                eprintln!("error syncing repo {}: {}", r.0.name, e);
            }
        });
    Ok(())
}

fn parse(cfg: &Config, cache_config: &RepoCacheOptions) -> Result<(), Box<Error>> {
    let elm_libs = elm_package::get_elm_libs()?;
    // try to parse each elm file
    let repo_exports: HashMap<String, Vec<ElmFile>> = HashMap::new();

    elm_libs
        .iter()
        .map(|i| (i, i.get_exports(&cache_config)))
        .fold(repo_exports, |mut repo_exports, res| match res.1 {
            Ok(file_res_vec) => {
                for file_res in file_res_vec {
                    match file_res {
                        Ok(elm_file) => {
                            println!(
                                "successfully parsed file {}: {:?}",
                                elm_file.path, elm_file.exports
                            );
                            match res.0.get_repo_path(cache_config) {
                                Ok(repo_path) => match repo_exports.get(&repo_path) {
                                    Some(elm_files) => {
                                        let mut new_elm_files = elm_files.clone();
                                        new_elm_files.push(elm_file);
                                        repo_exports
                                            .insert(res.0.name.to_string(), new_elm_files.to_vec());
                                    }
                                    None => {
                                        repo_exports.insert(res.0.name.to_string(), vec![elm_file]);
                                    }
                                },

                                ElmPackageError => {
                                    eprintln!("Something went wrong");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("error while parsing file: {}", e);
                        }
                    }
                }
                repo_exports
            }
            Err(e) => {
                eprintln!("error while trying to parse elm files: {}", e);
                repo_exports
            }
        })
        .iter()
        .for_each(
            |(name, exports)| match insert_functions(&cfg.db, name, exports) {
                Ok(()) => {
                    println!("Done inserting");
                }
                UpdateUrlError => eprintln!("something went wrong"),
            },
        );
    Ok(())
}

fn main() -> Result<(), Box<Error>> {
    let matches: ArgMatches = clap_app!(fn_search_backend_scrape =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CACHE_DIR: -d --("cache-dir") +takes_value +required "directory for repositories to be cached in")
        (@arg CHROME: -h --chrome +takes_value +required default_value("chromium") "google chrome or chromium executable")
        (@arg GIT: -g --git +takes_value +required default_value("git") "git executable")
        (@arg CONFIG: -c --config +takes_value +required "configuration file")
        (@subcommand sync =>
            (about: "sync repositories")
        )
        (@subcommand parse =>
            (about: "parse elm files")
        )
    ).get_matches();

    let cache_dir = matches
        .value_of("CACHE_DIR")
        .expect("error, no cache directory specified");
    let chrome = matches.value_of("CHROME").unwrap();
    let git = matches.value_of("GIT").unwrap();
    let config = matches.value_of("CONFIG").unwrap();
    let config = get_config(&config).map_err(|e| e as Box<Error>)?;
    let cache_config = RepoCacheOptions {
        cache_path: String::from(cache_dir),
        chromium_bin_path: chrome.to_string(),
        git_bin_path: git.to_string(),
    };
    if let Some(_) = matches.subcommand_matches("sync") {
        sync(&config, &cache_config)?;
    } else if let Some(_) = matches.subcommand_matches("parse") {
        parse(&config, &cache_config)?;
    } else {
        eprintln!("usage: fn_search_backend_scrape --help");
    }
    Ok(())
}
