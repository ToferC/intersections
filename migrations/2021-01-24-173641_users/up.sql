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
    role VARCHAR(32) NOT NULL DEFAULT 'user'
);

INSERT INTO users (id, user_uuid, hash, salt, email, user_name, slug, role)
VALUES
    (0,
    '71268fcd-72cc-4f83-9077-68e41980cddc',
    E'[36, 97, 114, 103, 111, 110, 50, 105, 36, 118, 61, 49, 57, 36, 109, 61, 52, 48, 57, 54, 44, 116, 61, 51, 44, 112, 61, 49, 36, 86, 86, 108, 119, 85, 71, 53, 105, 87, 108, 90, 52, 86, 84, 65, 106, 82, 83, 90, 114, 85, 87, 48, 111, 89, 84, 86, 81, 86, 71, 70, 51, 98, 105, 86, 116, 84, 67, 112, 54, 87, 85, 52, 108, 85, 121, 77, 120, 101, 71, 104, 119, 85, 85, 86, 118, 82, 69, 52, 50, 85, 51, 82, 97, 100, 69, 108, 90, 83, 84, 66, 112, 99, 87, 57, 107, 86, 108, 81, 108, 101, 109, 120, 84, 97, 70, 89, 53, 74, 87, 57, 81, 84, 68, 74, 80, 75, 70, 53, 72, 79, 68, 70, 72, 100, 67, 89, 107, 100, 108, 70, 54, 87, 72, 74, 107, 99, 71, 53, 52, 98, 71, 108, 72, 85, 107, 119, 49, 82, 71, 107, 122, 74, 106, 108, 122, 73, 86, 73, 50, 74, 84, 104, 82, 98, 107, 70, 111, 86, 48, 111, 112, 90, 88, 74, 106, 83, 109, 120, 72, 89, 122, 70, 75, 101, 69, 74, 68, 78, 84, 103, 36, 57, 79, 121, 103, 65, 120, 120, 111, 103, 113, 112, 54, 47, 115, 108, 72, 103, 73, 85, 84, 76, 70, 49, 57, 56, 86, 87, 85, 54, 114, 110, 117, 47, 111, 50, 98, 68, 86, 79, 107, 99, 111, 69]',
    'UYpPnbZVxU0#E&kQm(a5PTawn%mL*zYN%S#1xhpQEoDN6StZtIYI0iqodVT%zlShV9%oPL2O(^G81Gt&$vQzXrdpnxliGRL5Di3&9s!R6%8QnAhWJ)ercJlGc1JxBC58',
    'admin@admin.com',
    'Test Admin',
    'test_admin',
    'admin') RETURNING id as test_admin_id;

CREATE UNIQUE INDEX users__email_idx ON users(email);

-- Extend code to accomodate new format
ALTER TABLE people
    ALTER COLUMN code SET DATA TYPE VARCHAR(11);

-- Update communities table to add foreignkey to user_id
AlTER TABLE communities
    ADD COLUMN user_id INT NOT NULL DEFAULT 0,
    ADD CONSTRAINT user_id FOREIGN KEY(user_id)
        REFERENCES users(id)