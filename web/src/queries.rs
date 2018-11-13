use crate::collections::FnCache;
use fn_search_backend_db::diesel::{pg::PgConnection, result::QueryResult};

pub mod functions;

use crate::queries::functions::get_all_func_sigs;

pub fn make_fn_cache(conn: &PgConnection) -> QueryResult<FnCache> {
    Ok(get_all_func_sigs(conn)?.into_iter().collect())
}
