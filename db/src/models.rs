
use schema::*;

#[derive(Queryable)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name="repositories"]
pub struct NewRepository<'a> {
    pub name: &'a str,
    pub url: &'a str,
}

#[derive(Queryable)]
pub struct Function {
    pub id: i64,
    pub repo_id: i32,
    pub type_signature: String,
}

#[derive(Insertable)]
#[table_name="functions"]
pub struct NewFunction<'a> {
    pub repo_id: i32,
    pub type_signature: &'a str,
}
