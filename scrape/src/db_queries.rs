use crate::elm_package::ElmFile;
use fn_search_backend::DbConfig;
use fn_search_backend_parsers::ElmExport;
// import issues
use fn_search_backend_db::{
    diesel::{self, prelude::*, PgConnection},
    get_db_url,
    models::*,
    schema::*,
};
use rayon::prelude::*;
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

pub fn update_repo(
    cfg: &DbConfig,
    repo: &str,
    url: &str,
    version: &str,
) -> Result<(), UpdateUrlError> {
    let db_url = get_db_url(&cfg);
    let conn = PgConnection::establish(db_url.as_str())?;
    conn.transaction(|| -> Result<(), UpdateUrlError> {
        let new_repo = NewRepository {
            name: repo,
            url,
            ver: version,
        };
        let mut repos = repositories::table
            .filter(repositories::name.eq(&repo))
            .limit(1)
            .load::<Repository>(&conn)?;
        match repos.pop() {
            Some(mut repo) => {
                repo.url = url.to_string();
                repo.save_changes::<Repository>(&conn)?;
            }
            None => {
                diesel::insert_into(repositories::table)
                    .values(&new_repo)
                    .execute(&conn)?;
            }
        };
        Ok(())
    })?;
    Ok(())
}

pub fn insert_functions(
    cfg: &DbConfig,
    repo_name: &str,
    elm_files: &Vec<ElmFile>,
) -> Result<(), UpdateUrlError> {
    let db_url = get_db_url(&cfg);
    let conn = PgConnection::establish(db_url.as_str())?;
    let mut repos = repositories::table
        .filter(repositories::name.eq(&repo_name))
        .limit(1)
        .load::<Repository>(&conn)?;

    match repos.pop() {
        Some(repo) => {
            let exports: Vec<_> = elm_files
                .iter()
                .map(|file| file.exports.exports.iter())
                .flatten()
                .collect();
            let new_funcs: Vec<_> = exports
                .par_iter()
                .filter(|export| match export {
                    ElmExport::Function {..} => true,
                    _ => false,
                })
                .map(|export| match export {
                    ElmExport::Function {
                        name,
                        type_signature,
                    } => {
                        let a = match type_signature {
                            Some(typ_sig) => typ_sig.join(" "),
                            None => String::from(" "),
                        };
                        NewFunction {
                            repo_id: repo.id,
                            name: name.as_str(),
                            type_signature: a,
                        }
                    }
                    _ => panic!(),
                })
                .collect();
            diesel::insert_into(functions::table)
                .values(new_funcs.as_slice())
                .execute(&conn)?;
            Ok(())
        }
        None => {
            println!("No repository found {}", repo_name);
            Ok(())
        }
    }
}

pub fn refresh_repo_func_mat_view(cfg: &DbConfig) -> Result<(), Box<Error>> {
    let db_url = get_db_url(&cfg);
    let conn = PgConnection::establish(db_url.as_str())?;
    diesel::sql_query("REFRESH MATERIALIZED VIEW repository_function_mat_view").execute(&conn)?;
    Ok(())
}
