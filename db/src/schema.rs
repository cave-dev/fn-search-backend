table! {
    functions (id) {
        id -> Int8,
        repo_id -> Int4,
        name -> Text,
        type_signature -> Text,
    }
}

table! {
    repositories (id) {
        id -> Int4,
        name -> Text,
        url -> Text,
    }
}

joinable!(functions -> repositories (repo_id));

allow_tables_to_appear_in_same_query!(
    functions,
    repositories,
);
