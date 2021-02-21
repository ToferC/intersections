-- This file should undo anything in `up.sql`

ALTER TABLE communities
    DROP CONSTRAINT IF EXISTS user_id;

ALTER TABLE communities
    DROP COLUMN IF EXISTS user_id;

DROP TABLE users;
