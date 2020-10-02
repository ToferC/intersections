table! {
    use diesel::sql_types::*;
    use crate::models::Domain;

    domains (id) {
        id -> Int4,
        domain_token -> Domain,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::Domain;

    lenses (id) {
        id -> Int4,
        lens_name -> Varchar,
        date_created -> Timestamp,
        domain -> Varchar,
        inclusivity -> Int4,
        statements -> Nullable<Array<Text>>,
        person_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::Domain;

    persons (id) {
        id -> Int4,
        code -> Varchar,
        hashcode -> Varchar,
        date_created -> Timestamp,
    }
}

joinable!(lenses -> persons (person_id));

allow_tables_to_appear_in_same_query!(
    domains,
    lenses,
    persons,
);
