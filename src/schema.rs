table! {
    use diesel::sql_types::*;

    communities (id) {
        id -> Int4,
        tag -> Varchar,
        date_created -> Timestamp,
        code -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    lenses (id) {
        id -> Int4,
        node_name -> Varchar,
        node_domain -> Varchar,
        person_id -> Int4,
        node_id -> Int4,
        date_created -> Timestamp,
        statements -> Array<Text>,
        inclusivity -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;

    nodes (id) {
        id -> Int4,
        node_name -> Varchar,
        domain_token -> Varchar,
        translation -> Varchar,
        synonyms -> Array<Text>,
    }
}

table! {
    use diesel::sql_types::*;

    people (id) {
        id -> Int4,
        code -> Varchar,
        date_created -> Timestamp,
        related_codes -> Array<Text>,
        community_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    users (user_id) {
        user_id -> Int4,
        user_uuid -> Uuid,
        hash -> Bytea,
        salt -> Varchar,
        email -> Varchar,
        role -> Varchar,
        user_name -> Varchar,
        created_at -> Timestamp,
    }
}

joinable!(lenses -> nodes (node_id));
joinable!(lenses -> people (person_id));
joinable!(people -> communities (community_id));

allow_tables_to_appear_in_same_query!(
    communities,
    lenses,
    nodes,
    people,
    users,
);
