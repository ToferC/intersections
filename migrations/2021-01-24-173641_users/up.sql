-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    hash BYTEA NOT NULL,
    salt VARCHAR(255) NOT NULL,
    email VARCHAR(120) NOT NULL UNIQUE,
    user_name VARCHAR(32) NOT NULL UNIQUE,
    slug VARCHAR(32) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    role VARCHAR(32) NOT NULL DEFAULT 'user',
    managed_communities INT[] NOT NULL
);

CREATE UNIQUE INDEX users__email_idx ON users(email);

-- Extend code to accomodate new format
ALTER TABLE people
    ALTER COLUMN code SET DATA TYPE VARCHAR(11);