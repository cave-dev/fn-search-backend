table! {
    functions (id) {
        id -> Int8,
        repo_id -> Int4,
        type_signature -> Text,
        return_type -> Nullable<Text>,
    }
}

table! {
    repositories (id) {
        id -> Int4,
        url -> Text,
    }
}

joinable!(functions -> repositories (repo_id));

allow_tables_to_appear_in_same_query!(
    functions,
    repositories,
);
