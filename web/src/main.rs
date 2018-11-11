extern crate actix;
extern crate actix_web;
use actix_web::{
    server,
    App,
    HttpRequest,
    Result,
    Error,
    middleware::cors::Cors,
    error::{
        ErrorInternalServerError,
    }
};
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate radix_trie;
extern crate fn_search_backend;
extern crate fn_search_backend_db;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate parking_lot;
#[macro_use]
extern crate lazy_static;

pub(crate) mod collections;
pub(crate) mod app_state;
pub(crate) mod queries;
#[cfg(test)]
mod tests;

use actix_web::Responder;
use app_state::{AppState};
use fn_search_backend_db::utils::get_db_url;
use fn_search_backend::get_config;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use queries::make_fn_cache;
use std::sync::Arc;
use collections::FnCache;
use queries::functions::*;

fn search(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let sig: String = req.match_info().query("type_signature")?;
    let cache: Arc<FnCache> = req.state().get_fn_cache();
    let conn = req.state().db_conn().map_err(|e| ErrorInternalServerError(e))?;
    let res = (*cache).search(sig.as_str(), 10, None);
    Ok(match res {
        Some(ids) => {
            let funcs = get_functions(&conn, ids).map_err(|e| ErrorInternalServerError(e))?;
            serde_json::to_string(funcs.as_slice()).map_err(|e| ErrorInternalServerError(e))?
        },
        None => String::from("[]"),
    })
}

fn main() -> Result<(), Error> {
    let matches = clap_app!(fn_search_backend_web =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value +required "configuration file")
    ).get_matches();

    let cfg_file = matches.value_of("CONFIG").expect("error parsing configuration file");
    let cfg = get_config(&cfg_file).map_err(|e| ErrorInternalServerError(e))?;

    let pool = Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(get_db_url(&cfg.db))).map_err(|e| ErrorInternalServerError(e))?;

    let cache = Arc::new(make_fn_cache(&*pool.get().map_err(|e| ErrorInternalServerError(e))?).map_err(|e| ErrorInternalServerError(e))?);

    server::new(move || App::with_state(AppState::new(pool.clone(), cache.clone()))
        .configure(|app| {
            Cors::for_app(app)
                .allowed_origin("http://localhost:8080")
                .resource("/search/{type_signature}", |r| r.f(search))
                .register()
        }))
        .bind("127.0.0.1:8000")
        .unwrap()
        .run();
    Ok(())
}
