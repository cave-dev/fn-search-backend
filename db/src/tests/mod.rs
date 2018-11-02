
static RELATIVE_CFG_FILE: &'static str = "../config.toml";

use establish_connection;
use schema::*;
use models::*;
use diesel::prelude::*;
use diesel;

#[test]
fn insert_repo() {
    let connection = establish_connection(RELATIVE_CFG_FILE)
        .expect("error establishing connection to db");
    let new_repo = NewRepository{
        name: "cave-dev/fn-search-backend",
        url: "https://github.com/cave-dev/fn-search-backend",
    };

    diesel::insert_into(repositories::table)
        .values(&new_repo)
        // if our name is the same as one already in the db, update it
        .on_conflict(repositories::name)
        .do_update()
        .set(&new_repo)
        .get_result::<Repository>(&connection)
        .expect("Error saving new repo");
}

#[test]
fn test_db() {
    let connection = establish_connection(RELATIVE_CFG_FILE)
        .expect("error establishing connection to db");
    let results = repositories::table
        .limit(5)
        .load::<Repository>(&connection)
        .expect("Error loading repos");

    println!("Displaying {} repos", results.len());
    for repo in results {
        println!("{} -> {}", repo.id, repo.url);
    }
}
