-- This file should undo anything in `up.sql`

ALTER TABLE people
    DROP COLUMN IF EXISTS experiences;

ALTER TABLE nodes
    ALTER COLUMN domain_token TYPE VARCHAR(10),
    DROP CONSTRAINT IF EXISTS domain_token,
    ADD CONSTRAINT domain_token CHECK (domain_token IN ('person', 'role', 'system'));

ALTER TABLE experiences
    ALTER COLUMN node_domain TYPE VARCHAR(10),
    DROP CONSTRAINT IF EXISTS node_domain,
    ADD CONSTRAINT node_domain CHECK (node_domain IN ('person', 'role', 'system'));