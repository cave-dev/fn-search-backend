use diesel::{pg::PgConnection, prelude::*, result::QueryResult, Queryable};
use fn_search_backend_db::schema::*;
use serde_derive::Serialize;

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "repository_function_mat_view"]
pub struct CompleteFunction {
    pub repo_id: i32,
    pub repo_name: String,
    pub repo_url: String,
    pub func_id: i64,
    pub func_name: String,
    pub func_type_sig: String,
}

pub fn get_all_func_sigs(conn: &PgConnection) -> QueryResult<Vec<(String, i64)>> {
    use fn_search_backend_db::schema::functions::dsl::*;
    Ok(functions
        .select((type_signature, id))
        .load::<(String, i64)>(conn)?)
}

pub fn get_functions(conn: &PgConnection, ids: &[i64]) -> QueryResult<Vec<CompleteFunction>> {
    use fn_search_backend_db::schema::repository_function_mat_view::dsl::*;
    let fns = repository_function_mat_view
        .filter(func_id.eq_any(ids))
        .load::<CompleteFunction>(conn)?;
    Ok(fns)
}
