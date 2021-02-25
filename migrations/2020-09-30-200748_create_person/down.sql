-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS lenses;
DROP TABLE IF EXISTS nodes;
DROP TABLE IF EXISTS people;
DROP TABLE IF EXISTS communities CASCADE;
DROP TABLE IF EXISTS users;