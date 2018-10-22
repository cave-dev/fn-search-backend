#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use schema::*;
use models::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[test]
fn insert_repo() {
    let connection = establish_connection();

    let new_repo = NewRepository{
        url: String::from("https://github.com/cave-dev/fn-search-backend"),
    };

    diesel::insert_into(repositories::table)
        .values(&new_repo)
        .get_result::<Repository>(&connection)
        .expect("Error saving new repo");
}

#[test]
fn test_db() {
    let connection = establish_connection();
    let results = repositories::table
        .limit(5)
        .load::<Repository>(&connection)
        .expect("Error loading repos");

    println!("Displaying {} repos", results.len());
    for repo in results {
        println!("{} -> {}", repo.id, repo.url);
    }
}
