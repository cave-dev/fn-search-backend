#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use schema::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(Queryable)]
pub struct Repository {
    pub id: i32,
    pub url: String,
}

#[derive(Queryable)]
pub struct Function {
    pub id: i64,
    pub repo_id: i32,
    pub type_signature: String,
    pub return_type: Option<String>,
}

#[test]
fn test_db() {
    let connection = establish_connection();
    let results = repository
        .query()
        .limit(5)
        .load::<Repository>(&connection)
        .expect("Error loading repos");

    println!("Displaying {} repos", results.len());
    for repo in results {
        println!("{}", repo.id);
        println!("----------\n");
        println!("{}", repo.url);
    }
}
