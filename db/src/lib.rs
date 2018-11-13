#![allow(proc_macro_derive_resolution_fallback)]

//! ORM models for database interaction

#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate serde_derive;

pub mod schema;
pub mod models;
pub mod utils;

pub use crate::utils::{get_db_url, run_migrations};
