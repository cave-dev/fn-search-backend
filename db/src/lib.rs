#![allow(proc_macro_derive_resolution_fallback)]

//! ORM models for database interaction

extern crate clap;
#[macro_use]
pub extern crate diesel;
extern crate fn_search_backend;
extern crate diesel_migrations;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod schema;
pub mod models;
pub mod utils;

pub use utils::{get_db_url, run_migrations};
