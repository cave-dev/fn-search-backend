
use fn_search_backend_db::{get_db_url, models::*, schema::*, diesel::{self, PgConnection, prelude::*}};
use fn_search_backend::DbConfig;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UpdateUrlError {
    DieselConnectionError(diesel::result::ConnectionError),
    DieselError(diesel::result::Error),
    RepoNotFound,
}

impl Error for UpdateUrlError {}

impl fmt::Display for UpdateUrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<diesel::result::Error> for UpdateUrlError {
    fn from(e: diesel::result::Error) -> Self {
        UpdateUrlError::DieselError(e)
    }
}

impl From<diesel::result::ConnectionError> for UpdateUrlError {
    fn from(e: diesel::result::ConnectionError) -> Self {
        UpdateUrlError::DieselConnectionError(e)
    }
}

pub fn update_url(cfg: &DbConfig, repo: &str, url: &str) -> Result<(), UpdateUrlError> {
    let db_url = get_db_url(&cfg);
    let conn = PgConnection::establish(db_url.as_str())?;
    conn.transaction(|| -> Result<(), UpdateUrlError> {
        let new_repo = NewRepository{
            name: repo,
            url,
        };
        let res = diesel::insert_into(repositories::table)
            .values(&new_repo)
            // insert or do nothing
            .on_conflict(repositories::name)
            .do_nothing()
            .get_result::<Repository>(&conn).optional()?;
        // if we inserted something, we are done
        if let Some(_) = res {
            return Ok(());
        }
        let mut repos = repositories::table
            .filter(repositories::name.eq(&repo))
            .limit(1)
            .load::<Repository>(&conn)?;
        let mut repo = match repos.pop() {
            Some(r) => r,
            None => return Err(UpdateUrlError::RepoNotFound),
        };
        repo.url = url.to_string();
        repo.save_changes::<Repository>(&conn)?;
        Ok(())
    })?;
    Ok(())
}
