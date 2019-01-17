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

allow_tables_to_appear_in_same_query!(functions, repositories,);

table! {
    repository_function_mat_view (repo_id, func_id) {
        repo_id -> Integer,
        repo_name -> Text,
        repo_url -> Text,
        func_id -> BigInt,
        func_name -> Text,
        func_type_sig -> Text,
    }
}
