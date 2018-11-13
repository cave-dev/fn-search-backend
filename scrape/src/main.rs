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

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

pub mod elm_package;
pub mod repo_cache;

use std::error::Error;
use rayon::prelude::*;
use crate::repo_cache::{sync_repo, RepoCacheOptions};
use std::io::{stderr, Write};
use crate::elm_package::ElmPackageMetadataRaw;

fn main() -> Result<(), Box<Error>> {
    let matches = clap_app!(myapp =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CACHE_DIR: -d --("cache-dir") +takes_value +required "directory for repositories to be cached in")
    ).get_matches();

    let cfg_file = matches.value_of("CACHE_DIR").expect("error, no cache directory specified");
    let config = RepoCacheOptions{
        cache_path: String::from(cfg_file),
    };
    let elm_libs = elm_package::get_elm_libs()?;
    let cloned_libs: Vec<&ElmPackageMetadataRaw> = elm_libs
        .par_iter()
        .map(|i| {
            (i, sync_repo(i, &config))
        })
        .filter_map(|r| {
            match r.1 {
                Ok(_o) => {
                    // we can potentially do something with stdout & stderr of the clone process
                    println!("cloned repo {}", r.0.name);
                    Some(r.0)
                },
                Err(e) => {
                    let serr = stderr();
                    write!(serr.lock(), "{}\n", e);
                    None
                },
            }
        })
        .collect();
    println!("{:?}", cloned_libs);
    Ok(())
}
