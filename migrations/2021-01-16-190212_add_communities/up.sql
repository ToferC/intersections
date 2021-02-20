-- Your SQL goes here
CREATE TABLE communities (
    id SERIAL PRIMARY KEY,
    tag VARCHAR(32) NOT NULL,
    description VARCHAR NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    open BOOL NOT NULL DEFAULT FALSE,
    code VARCHAR(11) UNIQUE NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL
);

INSERT INTO communities (id, tag, description, open, code, slug)
VALUES
    (0, 'general', 'test community', FALSE, 'aif-oah-s77', 'test_community');

ALTER TABLE people
    ADD COLUMN community_id INT NOT NULL default 0;

ALTER TABLE people
ADD CONSTRAINT community_id FOREIGN KEY(community_id)
    REFERENCES communities(id) on DELETE CASCADE;