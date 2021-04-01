CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    hash BYTEA NOT NULL,
    salt VARCHAR(255) NOT NULL,
    email VARCHAR(128) NOT NULL UNIQUE,
    user_name VARCHAR(32) NOT NULL UNIQUE,
    slug VARCHAR(32) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    role VARCHAR(32) NOT NULL DEFAULT 'user'
);

CREATE UNIQUE INDEX users__email_idx ON users(email);

CREATE TABLE communities (
    id SERIAL PRIMARY KEY,
    tag VARCHAR(64) NOT NULL,
    description VARCHAR NOT NULL,
    data_use_case VARCHAR NOT NULL,
    contact_email VARCHAR(128) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    open BOOL NOT NULL DEFAULT FALSE,
    code VARCHAR(128) UNIQUE NOT NULL,
    slug VARCHAR(128) UNIQUE NOT NULL,
    user_id INT NOT NULL DEFAULT 0,
    FOREIGN KEY(user_id)
        REFERENCES users(id),
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
    node_name VARCHAR(32) UNIQUE NOT NULL,
    domain_token VARCHAR(10) NOT NULL CHECK (domain_token IN ('person', 'role', 'system')),
    translation varchar(32) NOT NULL default '',
    synonyms text[] NOT NULL default '{""}',
    slug VARCHAR(48) UNIQUE NOT NULL
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) NOT NULL,
    node_domain VARCHAR(10) NOT NULL CHECK (node_domain IN ('person', 'role', 'system')),
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES people(id) ON DELETE CASCADE,
    node_id INT NOT NULL,
    FOREIGN KEY(node_id)
        REFERENCES nodes(id),
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    statements TEXT[] NOT NULL,
    inclusivity NUMERIC NOT NULL
);