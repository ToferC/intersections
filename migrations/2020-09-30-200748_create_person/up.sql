CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    related_codes TEXT[] NOT NULL
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) UNIQUE NOT NULL,
    domain_token VARCHAR(10) NOT NULL CHECK (domain_token IN ('person', 'role', 'system'))
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) NOT NULL,
    node_domain VARCHAR(10) NOT NULL CHECK (node_domain IN ('person', 'role', 'system')),
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES people(id) ON DELETE CASCADE,
    node_id INT NOT NULL,
    FOREIGN KEY(node_id)
        REFERENCES nodes(id) on DELETE CASCADE,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    statements TEXT[] NOT NULL,
    inclusivity NUMERIC NOT NULL
);

INSERT INTO people (code, related_codes)
VALUES
    ('aifoahs89', array[]::text[]),
    ('syufioash', array[]::text[]),
    ('yueia8971', array[]::text[]),
    ('najksndfq', '{"aifoahs89"}');

INSERT INTO nodes (node_name, domain_token)
VALUES
    ('father', 'person'),
    ('manager', 'role'),
    ('gen x', 'person'),
    ('mother', 'person'),
    ('white', 'role'),
    ('black', 'role'),
    ('executive', 'role'),
    ('innovator', 'system');

INSERT INTO lenses (node_name, node_domain, person_id, node_id, statements, inclusivity)
VALUES
    ('father', 'person', 1, 1, '{"tired", "not doing enough", "joyful"}', -0.18),
    ('manager', 'role', 1, 2, '{"pulled many directions", "influential", "stressed"}', -0.25),
    ('gen x', 'person', 1, 3, '{"experienced", "overlooked", "depended upon"}', 0.23),
    ('mother', 'person', 2, 4, '{"tired", "guilty", "excluded"}', -0.45),
    ('white', 'person', 2, 5, '{"normal"}', 0.30),
    ('black', 'person', 3, 6, '{"ignored", "suffer microagressions", "proud"}', -0.30),
    ('mother', 'person', 3, 4, '{"balanced", "responsible", "capable"}', 0.29),
    ('executive', 'role',3, 7, '{"powerful", "overwhelmed", "stifled"}', -0.10),
    ('innovator', 'system', 3, 8, '{"respected", "sidelined", "not recognized by system"}', 0.20),
    ('white', 'person', 4, 5, '{"listened to", "persecuted by diversity iniatives", "comfortable"}', 0.09);
