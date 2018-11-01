
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate rayon;
#[macro_use]
extern crate clap;

pub mod elm_package;
pub mod elm_parse;
pub mod repo_cache;

use std::error::Error;
use rayon::prelude::*;
use repo_cache::{sync_repo, RepoCacheOptions};
use std::io::{stderr, Write};
use elm_package::ElmPackageMetadata;

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
    let cloned_libs: Vec<&ElmPackageMetadata> = elm_libs
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
