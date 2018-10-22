
use schema::*;

#[derive(Queryable)]
pub struct Repository {
    pub id: i32,
    pub url: String,
}

#[derive(Insertable)]
#[table_name="repositories"]
pub struct NewRepository {
    pub url: String,
}

#[derive(Queryable)]
pub struct Function {
    pub id: i64,
    pub repo_id: i32,
    pub type_signature: String,
}

#[derive(Insertable)]
#[table_name="functions"]
pub struct NewFunction {
    pub repo_id: i32,
    pub type_signature: String,
}
