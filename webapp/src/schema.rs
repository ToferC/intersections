table! {
    use diesel::sql_types::*;

    communities (id) {
        id -> Int4,
        tag -> Int4,
        description -> Int4,
        data_use_case -> Int4,
        contact_email -> Varchar,
        date_created -> Timestamp,
        open -> Bool,
        code -> Varchar,
        slug -> Varchar,
        user_id -> Int4,
        data -> Jsonb,
        test -> Bool,
    }
}

table! {
    use diesel::sql_types::*;

    email_verification_code (id) {
        id -> Int4,
        email_address -> Varchar,
        activation_code -> Varchar,
        expires_on -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    experiences (id) {
        id -> Int4,
        node_name -> Int4,
        node_domain -> Varchar,
        person_id -> Int4,
        node_id -> Int4,
        date_created -> Timestamp,
        statements -> Array<Int4>,
        inclusivity -> Numeric,
        slug -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    nodes (id) {
        id -> Int4,
        node_name -> Int4,
        domain_token -> Varchar,
        synonyms -> Array<Int4>,
        slug -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    password_reset_token (id) {
        id -> Int4,
        email_address -> Varchar,
        reset_token -> Varchar,
        expires_on -> Timestamp,
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
        experiences -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    phrases (id, lang) {
        id -> Int4,
        lang -> Varchar,
        text -> Varchar,
        machine_translation -> Bool,
    }
}

table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        user_uuid -> Uuid,
        hash -> Bytea,
        salt -> Varchar,
        email -> Varchar,
        user_name -> Varchar,
        slug -> Varchar,
        created_at -> Timestamp,
        role -> Varchar,
        validated -> Bool,
    }
}

joinable!(communities -> users (user_id));
joinable!(experiences -> nodes (node_id));
joinable!(experiences -> people (person_id));
joinable!(people -> communities (community_id));

allow_tables_to_appear_in_same_query!(
    communities,
    email_verification_code,
    experiences,
    nodes,
    password_reset_token,
    people,
    phrases,
    users,
);
