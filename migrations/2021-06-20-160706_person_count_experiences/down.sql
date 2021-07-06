-- This file should undo anything in `up.sql`

ALTER TABLE people
    DROP COLUMN IF EXISTS experiences;