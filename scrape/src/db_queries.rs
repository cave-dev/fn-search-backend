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
        let new_repo = NewRepository { name: repo, url };
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
            conn.transaction(|| -> Result<(), UpdateUrlError> {
                elm_files
                    .iter()
                    .map(|file| file.exports.exports.iter())
                    .flatten()
                    .for_each(|export| match export {
                        // matching enums
                        ElmExport::Function {
                            name: name,
                            type_signature: type_signature,
                        } => {
                            let a = match type_signature {
                                Some(typ_sig) => typ_sig.join(" "),
                                None => String::from(" "),
                            };
                            println!("inserting to db: {}: {:?}", name, type_signature);
                            let new_function = NewFunction {
                                repo_id: repo.id,
                                name: name,
                                type_signature: &a,
                            };
                            diesel::insert_into(functions::table)
                                .values(&new_function)
                                .execute(&conn);
                        }
                        ElmExport::Type {
                            name: name,
                            definition: definition,
                        } => {
                            let new_function = NewFunction {
                                repo_id: repo.id,
                                name: name,
                                type_signature: definition,
                            };
                            println!("inserting to db: {}: {:?}", name, definition);
                            diesel::insert_into(functions::table)
                                .values(&new_function)
                                .execute(&conn);
                        }
                    });
                Ok(())
            })
        }
        None => {
            println!("No repository found {}", repo_name);
            // no repository found
            Ok(())
        }
    };
    Ok(())
}
