table! {
    use diesel::sql_types::*;

    lenses (id) {
        id -> Int4,
        person_id -> Int4,
        node_id -> Int4,
        date_created -> Timestamp,
        statements -> Nullable<Array<Text>>,
        inclusivity -> Numeric,
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

    people (id) {
        id -> Int4,
        code -> Varchar,
        date_created -> Timestamp,
        related_codes -> Array<Text>,
    }
}

joinable!(lenses -> nodes (node_id));
joinable!(lenses -> people (person_id));

allow_tables_to_appear_in_same_query!(
    lenses,
    nodes,
    people,
);
