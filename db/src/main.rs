#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate fn_search_backend;
extern crate diesel_migrations;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod schema;
pub mod models;
pub mod utils;

use diesel::pg::PgConnection;
use diesel::Connection;
use crate::utils::{get_db_url, run_migrations};
use std::error::Error;
use fn_search_backend::get_config;

fn main() -> Result<(), Box<Error>> {
    let matches = clap_app!(fn_search_backend_db =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value +required "configuration file")
    ).get_matches();

    let cfg_file = matches.value_of("CONFIG").expect("error parsing configuration file");
    let cfg = get_config(cfg_file)
        .expect("error getting configuration file");
    let conn = PgConnection::establish(&get_db_url(&cfg.db)).map_err(|e| {
        println!("error while running migrations: {}", e);
        ()
    }).unwrap(); // panic on error
    run_migrations(&conn).map_err(|e| {
        println!("error while running migrations: {}", e);
        ()
    }).unwrap(); // panic on error
    Ok(())
}
