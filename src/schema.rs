table! {
    use diesel::sql_types::*;

    lenses (id) {
        id -> Int4,
        lens_name -> Varchar,
        date_created -> Timestamp,
        domain_token -> Varchar,
        inclusivity -> Numeric,
        statements -> Nullable<Array<Text>>,
        person_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    persons (id) {
        id -> Int4,
        code -> Varchar,
        hashcode -> Varchar,
        date_created -> Timestamp,
    }
}

joinable!(lenses -> persons (person_id));

allow_tables_to_appear_in_same_query!(
    lenses,
    persons,
);
