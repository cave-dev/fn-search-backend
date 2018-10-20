table! {
    function (id) {
        id -> Int8,
        repo_id -> Int4,
        type_signature -> Text,
        return_type -> Nullable<Text>,
    }
}

table! {
    repository (id) {
        id -> Int4,
        url -> Text,
    }
}

joinable!(function -> repository (repo_id));

allow_tables_to_appear_in_same_query!(
    function,
    repository,
);
