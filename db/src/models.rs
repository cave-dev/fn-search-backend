use crate::schema::*;
use serde_derive::Serialize;

#[derive(Queryable, Clone, Debug, Serialize, Deserialize, AsChangeset, Identifiable)]
#[table_name = "repositories"]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub ver: String,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "repositories"]
pub struct NewRepository<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub ver: &'a str,
}

#[derive(Queryable, Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub id: i64,
    pub repo_id: i32,
    pub name: String,
    pub type_signature: String,
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        self.id == other.id
    }
}

#[derive(Insertable, Debug)]
#[table_name = "functions"]
pub struct NewFunction<'a> {
    pub repo_id: i32,
    pub type_signature: String,
    pub name: &'a str,
}

#[derive(Serialize, Queryable, QueryableByName)]
#[table_name = "repository_function_mat_view"]
pub struct FunctionWithRepo {
    pub repo_id: i32,
    pub repo_name: String,
    pub repo_url: String,
    pub repo_version: String,
    pub func_id: i64,
    pub func_name: String,
    pub func_type_sig: String,
}
