#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate fn_search_backend;
extern crate diesel_migrations;

mod schema;
mod models;
mod utils;

use utils::{establish_connection, run_migrations};

fn main() {
    let matches = clap_app!(fn_search_backend_db =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value +required "configuration file")
    ).get_matches();

    let cfg_file = matches.value_of("CONFIG").expect("error parsing configuration file");
    let conn = establish_connection(cfg_file).map_err(|e| {
        println!("error while running migrations: {}", e);
        ()
    }).unwrap(); // panic on error
    run_migrations(&conn).map_err(|e| {
        println!("error while running migrations: {}", e);
        ()
    }).unwrap(); // panic on error
}
