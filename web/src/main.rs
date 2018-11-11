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
    },
    middleware::Logger,
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
extern crate env_logger;

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

fn suggest(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let sig: String = req.match_info().query("type_signature")?;
    let cache: Arc<FnCache> = req.state().get_fn_cache();
    let res = (*cache).suggest(sig.as_str(), 10);
    Ok(match res {
        Some(sigs) => {
            serde_json::to_string(sigs.as_slice()).map_err(|e| ErrorInternalServerError(e))?
        },
        None => String::from("[]")
    })
}

fn update_fns(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let conn = req.state().db_conn().map_err(|e| ErrorInternalServerError(e))?;
    let sigs = get_all_func_sigs(&(*conn)).map_err(|e| ErrorInternalServerError(e))?;
    let fn_cache: FnCache = sigs.into_iter().collect();
    req.state().update_fn_cache(fn_cache);
    Ok("OK")
}

fn main() {
    let matches: clap::ArgMatches = clap_app!(fn_search_backend_web =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value +required "configuration file")
        (@arg VERBOSITY: -v +multiple "Sets verbosity level.\n-v : error\n-vv : info\n-vvv : debug")
    ).get_matches();

    let log_str = match matches.occurrences_of("VERBOSITY") {
        1 => "actix_web=error",
        2 => "actix_web=info",
        3 => "actix_web=debug",
        0 => "",
        _ => panic!("unknown log level"),
    };

    if log_str != "" {
        std::env::set_var("RUST_LOG", log_str);
    }
    env_logger::init();

    let cfg_file = matches.value_of("CONFIG").expect("error parsing configuration file");
    let cfg = get_config(&cfg_file).expect("error loading configuration file");
    let cfg_arc = Arc::new(cfg);

    let pool = Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(get_db_url(&cfg_arc.clone().db))).expect("error setting up database connection");

    let fn_cache = make_fn_cache(&*pool.get().expect("error connecting to database"));
    let cache = Arc::new(fn_cache.expect("error retrieving function type signatures"));

    let cfg_clone = cfg_arc.clone();
    server::new(move || App::with_state(AppState::new(pool.clone(), cache.clone()))
        .configure(|app| {
            Cors::for_app(app)
                .allowed_origin(&cfg_arc.web.allowed_origin)
                .resource("/search/{type_signature}", |r| r.f(search))
                .resource("/suggest/{type_signature}", |r| r.f(search))
                .resource("/update_functions", |r| r.f(update_fns))
                .register()
                .middleware(Logger::default())
        }))
        .bind(&cfg_clone.web.bind_address)
        .unwrap()
        .run();
}
