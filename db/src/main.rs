#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate fn_search_backend;
extern crate diesel_migrations;

pub mod schema;
pub mod models;
#[cfg(test)]
mod tests;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel_migrations::run_pending_migrations;
use fn_search_backend::get_config;
use std::error::Error;

pub fn establish_connection(f: &str) -> Result<PgConnection, Box<Error>> {
    let cfg = get_config(f)?.db;
    let database_url = format!("postgres://{}:{}@{}/{}", cfg.user, cfg.password, cfg.host, cfg.db);
    Ok(PgConnection::establish(&database_url)?)
}

pub fn run_migrations(conn: &PgConnection) -> Result<(), Box<Error>> {
    run_pending_migrations(conn)?;
    Ok(())
}

fn main() {
    let matches = clap_app!(myapp =>
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
