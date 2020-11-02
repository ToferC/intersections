-- This file should undo anything in `up.sql`

DELETE from nodes
WHERE node_name IN (
    ' black');