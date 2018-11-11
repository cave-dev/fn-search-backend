use schema::*;

#[derive(Queryable, Clone, Debug, Serialize, Deserialize)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name="repositories"]
pub struct NewRepository<'a> {
    pub name: &'a str,
    pub url: &'a str,
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
#[table_name="functions"]
pub struct NewFunction<'a> {
    pub repo_id: i32,
    pub type_signature: &'a str,
}
