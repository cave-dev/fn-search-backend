use fn_search_backend_db::{
    diesel::pg::PgConnection, diesel::prelude::*, diesel::result::QueryResult, models::*,
};

pub fn get_all_func_sigs(conn: &PgConnection) -> QueryResult<Vec<(String, i64)>> {
    use fn_search_backend_db::schema::functions::dsl::*;
    Ok(functions
        .select((type_signature, id))
        .load::<(String, i64)>(conn)?)
}

pub fn get_functions(conn: &PgConnection, ids: &[i64]) -> QueryResult<Vec<Function>> {
    use fn_search_backend_db::schema::functions::dsl::*;
    let fns = functions.filter(id.eq_any(ids)).load::<Function>(conn)?;
    Ok(fns)
}
