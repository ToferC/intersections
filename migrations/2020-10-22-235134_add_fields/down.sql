-- This file should undo anything in `up.sql`

ALTER TABLE nodes
DROP COLUMN translation;

ALTER TABLE nodes
DROP COLUMN synonyms;