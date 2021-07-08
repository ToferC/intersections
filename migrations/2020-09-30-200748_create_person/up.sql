CREATE TABLE phrases (
    id SERIAL NOT NULL,
    lang VARCHAR(2) NOT NULL,
    text VARCHAR NOT NULL,
    machine_translation bool NOT NULL default false,
    PRIMARY KEY(id, lang)
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    hash BYTEA NOT NULL,
    salt VARCHAR(255) NOT NULL,
    email VARCHAR(128) NOT NULL UNIQUE,
    user_name VARCHAR(32) NOT NULL UNIQUE,
    slug VARCHAR(32) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    role VARCHAR(32) NOT NULL DEFAULT 'user',
    validated bool NOT NULL DEFAULT false
);

CREATE UNIQUE INDEX users__email_idx ON users(email);

CREATE TABLE communities (
    id SERIAL PRIMARY KEY,
    tag INT NOT NULL,
    description INT NOT NULL,
    data_use_case INT NOT NULL,
    contact_email VARCHAR(128) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    open BOOL NOT NULL DEFAULT FALSE,
    code VARCHAR(128) UNIQUE NOT NULL,
    slug VARCHAR(128) UNIQUE NOT NULL,
    user_id INT NOT NULL DEFAULT 0,
    FOREIGN KEY(user_id)
        REFERENCES users(id) ON DELETE CASCADE,
    data JSONB NOT NULL,
    test BOOL NOT NULL DEFAULT FALSE
);

CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    code VARCHAR(128) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    related_codes TEXT[] NOT NULL,
    community_id INT NOT NULL,
    FOREIGN KEY(community_id)
        REFERENCES communities(id) on DELETE CASCADE
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    node_name INT UNIQUE NOT NULL,
    domain_token VARCHAR(32) NOT NULL CHECK (domain_token IN (
        'race_culture',
        'gender',
        'sexuality',
        'socio_economic',
        'language',
        'education',
        'religion',
        'ability_disability',
        'personality',
        'age',
        'mental_health',
        'body_image',
        'relationship_caregiving',
        'employment_status',
        'organizational_role',
        'community_role',
        'other'
    )),
    synonyms INT[] NOT NULL default '{}',
    slug VARCHAR(48) UNIQUE NOT NULL
);

CREATE TABLE experiences (
    id SERIAL PRIMARY KEY,
    node_name INT NOT NULL,
    node_domain VARCHAR(32) NOT NULL CHECK (node_domain IN (
        'race_culture',
        'gender',
        'sexuality',
        'socio_economic',
        'language',
        'education',
        'religion',
        'ability_disability',
        'personality',
        'age',
        'mental_health',
        'body_image',
        'relationship_caregiving',
        'employment_status',
        'organizational_role',
        'community_role',
        'other'
    )),
    importance INT NOT NULL DEFAULT 3,
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES people(id) ON DELETE CASCADE,
    node_id INT NOT NULL,
    FOREIGN KEY(node_id)
        REFERENCES nodes(id),
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    statements INT[] NOT NULL,
    inclusivity NUMERIC NOT NULL,
    slug VARCHAR(48) NOT NULL
);