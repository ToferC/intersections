table! {
    use diesel::sql_types::*;

    lenses (id) {
        id -> Int4,
        date_created -> Timestamp,
        inclusivity -> Numeric,
        statements -> Nullable<Array<Text>>,
        node_id -> Int4,
        person_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    nodes (id) {
        id -> Int4,
        node_name -> Varchar,
        domain_token -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    persons (id) {
        id -> Int4,
        code -> Varchar,
        hash_code -> Varchar,
        date_created -> Timestamp,
    }
}

joinable!(lenses -> nodes (node_id));
joinable!(lenses -> persons (person_id));

allow_tables_to_appear_in_same_query!(
    lenses,
    nodes,
    persons,
);
