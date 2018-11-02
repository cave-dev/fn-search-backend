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
