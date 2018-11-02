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
pub mod utils;

pub use utils::{establish_connection, run_migrations};
