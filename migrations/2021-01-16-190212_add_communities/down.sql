-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS communities CASCADE;

ALTER TABLE people
    DROP CONSTRAINT IF EXISTS community_id;

ALTER TABLE people
    DROP COLUMN IF EXISTS community_id;


