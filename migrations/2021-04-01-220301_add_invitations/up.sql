-- Your SQL goes here
/*
CREATE TABLE community_invitations (
    id UUID PRIMARY KEY,
    community_id INT NOT NULL,
    FOREIGN KEY(community_id)
        REFERENCES communities(id) ON DELETE CASCADE,
    code VARCHAR(128) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    expires_on TIMESTAMP NOT NULL
);
*/

CREATE TABLE IF NOT EXISTS email_verification_code (
    id SERIAL PRIMARY KEY,
    email_address VARCHAR(64) UNIQUE NOT NULL,
    activation_code VARCHAR(5) UNIQUE NOT NULL,
    expires_on TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS password_reset_token (
    id SERIAL PRIMARY KEY,
    email_address VARCHAR(64) UNIQUE NOT NULL,
    reset_token VARCHAR(36) UNIQUE NOT NULL,
    expires_on TIMESTAMP NOT NULL
);