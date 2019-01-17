#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;

// use jemallocator as our allocator
extern crate jemallocator;
use jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub(crate) mod app_state;
pub(crate) mod collections;
pub(crate) mod queries;
#[cfg(test)]
mod tests;

use crate::app_state::AppState;
use crate::collections::FnCache;
use crate::queries::functions::*;
use crate::queries::make_fn_cache;
use actix_web::Responder;
use actix_web::{
    error::ErrorInternalServerError, middleware::cors::Cors, middleware::Logger, server, App,
    HttpRequest, Result,
};
use fn_search_backend::get_config;
use fn_search_backend_db::utils::get_db_url;
use percent_encoding::percent_decode;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::sync::Arc;

fn search(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let sig: String = req.match_info().query("type_signature")?;
    let sig = percent_decode(sig.as_bytes()).decode_utf8()?.to_string();
    let cache: Arc<FnCache> = req.state().get_fn_cache();
    let conn = req
        .state()
        .db_conn()
        .map_err(|e| ErrorInternalServerError(e))?;
    let res = (*cache).search(sig.as_str(), 10, None);
    Ok(match res {
        Some(ids) => {
            let funcs = get_functions(&conn, ids).map_err(|e| ErrorInternalServerError(e))?;
            serde_json::to_string(funcs.as_slice())?
        }
        None => String::from("[]"),
    })
}

fn suggest(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let sig: String = req.match_info().query("type_signature")?;
    let sig = percent_decode(sig.as_bytes()).decode_utf8()?.to_string();
    let cache: Arc<FnCache> = req.state().get_fn_cache();
    let res = (*cache).suggest(sig.as_str(), 10);
    Ok(match res {
        Some(sigs) => serde_json::to_string(sigs.as_slice())?,
        None => String::from("[]"),
    })
}

fn update_fns(req: &HttpRequest<AppState>) -> Result<impl Responder> {
    let sigs = {
        let conn = req
            .state()
            .db_conn()
            .map_err(|e| ErrorInternalServerError(e))?;
        get_all_func_sigs(&(*conn)).map_err(|e| ErrorInternalServerError(e))?
    }; // database connection goes out of scope, returning to pool
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
    )
    .get_matches();

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

    let cfg_file = matches
        .value_of("CONFIG")
        .expect("error parsing configuration file");
    let cfg = get_config(&cfg_file).expect("error loading configuration file");
    let cfg = Arc::new(cfg);

    let pool = Pool::builder()
        .max_size(cfg.web.db_pool_size)
        .build(ConnectionManager::new(get_db_url(&cfg.clone().db)))
        .expect("error setting up database connection");

    let fn_cache = make_fn_cache(&*pool.get().expect("error connecting to database"));
    let cache = Arc::new(fn_cache.expect("error retrieving function type signatures"));

    let cfg_clone = cfg.clone();
    server::new(move || {
        App::with_state(AppState::new(pool.clone(), cache.clone())).configure(|app| {
            Cors::for_app(app)
                .allowed_origin(&cfg.web.allowed_origin)
                .resource("/search/{type_signature}", |r| r.f(search))
                .resource("/suggest/{type_signature}", |r| r.f(suggest))
                .resource("/update_functions", |r| r.f(update_fns))
                .register()
                .middleware(Logger::default())
        })
    })
    .bind(&cfg_clone.web.bind_address)
    .unwrap()
    .run();
}
