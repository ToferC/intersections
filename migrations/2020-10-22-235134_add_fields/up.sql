-- Your SQL goes here

ALTER TABLE nodes
ADD COLUMN translation varchar(32) NOT NULL default '';

ALTER TABLE nodes
ADD COLUMN synonyms text[] NOT NULL default '{""}';