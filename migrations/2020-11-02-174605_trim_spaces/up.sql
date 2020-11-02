-- Your SQL goes here

UPDATE nodes
SET node_name = TRIM (node_name)
ON CONFLICT (node_name)
    DO NOTHING;