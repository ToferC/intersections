-- Your SQL goes here
ALTER TABLE people 
    add column experiences INT NOT NULL DEFAULT 1;

ALTER TABLE nodes
    ALTER COLUMN domain_token TYPE VARCHAR(24),
    DROP CONSTRAINT IF EXISTS domain_token,
    ADD CONSTRAINT domain_token CHECK (domain_token IN (
        'race_culture',
        'gender',
        'sexuality',
        'socio_economic',
        'language',
        'education',
        'religion',
        'ability_disability',
        'personality',
        'age',
        'mental_health',
        'body_image',
        'relationship_caregiving',
        'employment_status',
        'organizational_role',
        'community_role',
        'other',
        'person', 'role', 'system'));

ALTER TABLE experiences
    ALTER COLUMN node_domain TYPE VARCHAR(24),
    DROP CONSTRAINT IF EXISTS node_domain,
    ADD CONSTRAINT node_domain CHECK (node_domain IN (
        'race_culture',
        'gender',
        'sexuality',
        'socio_economic',
        'language',
        'education',
        'religion',
        'ability_disability',
        'personality',
        'age',
        'mental_health',
        'body_image',
        'relationship_caregiving',
        'employment_status',
        'organizational_role',
        'community_role',
        'other',
        'person', 'role', 'system'));
