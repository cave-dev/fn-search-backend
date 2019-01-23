use diesel::{pg::PgConnection, prelude::*, result::QueryResult};
use fn_search_backend_db::models::FunctionWithRepo;

pub fn get_all_func_sigs(conn: &PgConnection) -> QueryResult<Vec<(String, i64)>> {
    use fn_search_backend_db::schema::functions::dsl::*;
    Ok(functions
        .select((type_signature, id))
        .load::<(String, i64)>(conn)?)
}

pub fn get_functions(conn: &PgConnection, ids: &[i64]) -> QueryResult<Vec<FunctionWithRepo>> {
    use fn_search_backend_db::schema::repository_function_mat_view::dsl::*;
    let fns = repository_function_mat_view
        .filter(func_id.eq_any(ids))
        .load::<FunctionWithRepo>(conn)?;
    Ok(fns)
}
