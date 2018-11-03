extern crate actix_web;
use actix_web::{
    server,
    App,
    HttpRequest,
    Result,
    middleware::cors::Cors,
};

#[macro_use]
extern crate lazy_static;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate radix_trie;
extern crate parking_lot;
extern crate fn_search_backend_db;

use rand::random;
use actix_web::Responder;
pub mod fn_cache;
#[cfg(test)]
mod tests;

lazy_static! {
    static ref SAMPLE_TYPES: Vec<&'static str> = vec![
        "String",
        "Int",
        "Bool",
        "List",
        "Time"
    ];
    static ref SAMPLE_SITES: Vec<&'static str> = vec![
        "https://github.com/",
        "https://bitbucket.com/",
        "https://gitlab.com/"
    ];
    static ref SAMPLE_USERS: Vec<&'static str> = vec![
        "bubby",
        "ffrancis",
        "captain_oblivious",
        "xXx_fortnitememes_xXx",
        "koopa",
        "derpington",
        "pecan"
    ];
    static ref SAMPLE_REPO_NAMES: Vec<&'static str> = vec![
        "awesomefifo",
        "epic_javascript_library",
        "NoJs",
        "pls",
        "AlT-cAsE-oNlY-cAsE",
        "derp"
    ];
    static ref SAMPLE_FUNCTION_NAMES: Vec<&'static str> = vec![
        "test_fn",
        "bogosort",
        "derpyfunc",
        "dothings",
        "FUnCtiOn_stUUUF"
    ];
}

static MAX_SAMPLE_TYPES: usize = 10;
static MAX_NUMBER_RESULTS: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
struct SearchResultRepo {
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResultFn {
    name: String,
    desc: String,
    args: Vec<String>,
    ret: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResult {
    repo: SearchResultRepo,
    res: SearchResultFn,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResultWrapper {
    data: Vec<SearchResult>,
}

fn gen_repo_url() -> (String, String) {
    let mut res = String::new();
    let url_index: usize = random::<usize>() % SAMPLE_SITES.len();
    res.push_str(SAMPLE_SITES[url_index]);
    let user_index: usize = random::<usize>() % SAMPLE_USERS.len();
    res.push_str(SAMPLE_USERS[user_index]);
    res.push('/');
    let repo_index: usize = random::<usize>() % SAMPLE_REPO_NAMES.len();
    res.push_str(SAMPLE_REPO_NAMES[repo_index]);
    (String::from(SAMPLE_REPO_NAMES[repo_index]), res)
}

fn gen_rand_search_result() -> String {
    let mut results: Vec<SearchResult> = Vec::new();
    let num_results: usize = random::<usize>() % MAX_NUMBER_RESULTS;
    for _ in 0..num_results {
        let mut types: Vec<String> = Vec::new();
        let num_types: usize = random::<usize>() % MAX_SAMPLE_TYPES;
        for _ in 0..num_types {
            let type_index: usize = random::<usize>() % SAMPLE_TYPES.len();
            types.push(String::from(SAMPLE_TYPES[type_index]));
        }
        let type_index: usize = random::<usize>() % SAMPLE_TYPES.len();
        let fn_index: usize = random::<usize>() % SAMPLE_FUNCTION_NAMES.len();
        let res_fn = SearchResultFn{
            name: String::from(SAMPLE_FUNCTION_NAMES[fn_index]),
            desc: String::from(SAMPLE_FUNCTION_NAMES[fn_index]),
            args: types,
            ret: String::from(SAMPLE_TYPES[type_index]),
        };
        let (repo_name, repo_url) = gen_repo_url();
        let res_repo = SearchResultRepo{
            name: repo_name,
            url: repo_url,
        };
        results.push(SearchResult{
            repo: res_repo,
            res: res_fn,
        })
    }
    let res_wrapper = SearchResultWrapper{
        data: results,
    };
    serde_json::to_string(&res_wrapper).unwrap()
}

fn index(req: &HttpRequest) -> Result<impl Responder> {
    let _: String = req.match_info().query("type_signature")?;
    Ok(gen_rand_search_result())
}

fn main() {
    server::new(move || App::new()
                .configure(|app| {
                    Cors::for_app(app)
                        .allowed_origin("http://localhost:8080")
                        .resource("/search/{type_signature}", |r| r.f(index))
                        .register()
    }))
        .bind("127.0.0.1:8000")
        .unwrap()
        .run();
}
