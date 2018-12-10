use diesel::prelude::*;
use diesel_migrations::{run_pending_migrations, RunMigrationsError};
use fn_search_backend::DbConfig;

pub fn get_db_url(cfg: &DbConfig) -> String {
    format!(
        "postgres://{}:{}@{}/{}",
        cfg.user, cfg.password, cfg.host, cfg.db
    )
}

/// run pending migrations
pub fn run_migrations(conn: &PgConnection) -> Result<(), RunMigrationsError> {
    run_pending_migrations(conn)?;
    Ok(())
}
