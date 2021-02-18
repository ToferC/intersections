-- Your SQL goes here
CREATE TABLE communities (
    id SERIAL PRIMARY KEY,
    tag VARCHAR(32) NOT NULL,
    description VARCHAR NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    code VARCHAR(11) NOT NULL
);

INSERT INTO communities (id, tag, description, code)
VALUES
    (0, 'general', 'test community', 'aif-oah-s77');

ALTER TABLE people
ADD COLUMN community_id INT NOT NULL default 0;

ALTER TABLE people
ADD CONSTRAINT community_id FOREIGN KEY(community_id)
    REFERENCES communities(id) on DELETE CASCADE;