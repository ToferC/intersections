CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    related_codes TEXT[] NOT NULL
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) UNIQUE NOT NULL,
    domain_token VARCHAR(10) NOT NULL CHECK (domain_token IN ('Person', 'Role', 'System'))
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) NOT NULL,
    node_domain VARCHAR(10) NOT NULL CHECK (node_domain IN ('Person', 'Role', 'System')),
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

INSERT INTO people (id, code, related_codes)
VALUES
    (1, 'aifoahs89', array[]::text[]),
    (2, 'syufioash', array[]::text[]),
    (3, 'yueia8971', array[]::text[]),
    (4, 'najksndfq', '{"aifoahs89"}');

INSERT INTO nodes (id, node_name, domain_token)
VALUES
    (1, 'Father', 'Person'),
    (2, 'Manager', 'Role'),
    (3, 'Gen X', 'Person'),
    (4, 'Mother', 'Person'),
    (5, 'White', 'Role'),
    (6, 'Black', 'Role'),
    (7, 'Executive', 'Role'),
    (8, 'Innovator', 'System');

INSERT INTO lenses (id, node_name, node_domain, person_id, node_id, statements, inclusivity)
VALUES
    (1, 'Father', 'Person', 1, 1, '{"Tired", "Not doing enough", "Joyful"}', -0.18),
    (2, 'Manager', 'Role', 1, 2, '{"Pulled many directions", "Influential", "Stressed"}', -0.38),
    (3, 'Gen X', 'Person', 1, 3, '{"Experienced", "Overlooked", "Depended upon"}', 0.28),
    (4, 'Mother', 'Person', 2, 4, '{"Tired", "Guilty", "Excluded"}', -0.33),
    (5, 'White', 'Person', 2, 5, '{"Normal"}', 0.28),
    (6, 'Black', 'Person', 3, 6, '{"Ignored", "Suffer microagressions", "Proud"}', -0.25),
    (7, 'Executive', 'Role',3, 7, '{"Powerful", "Overwhelmed", "Stifled"}', -0.10),
    (8, 'Innovator', 'System', 3, 8, '{"Respected", "Sidelined", "Not recognized by system"}', 0.28),
    (9, 'White', 'Person', 4, 5, '{"Listened to", "Persecuted by diversity iniatives", "Comfortable"}', 0.18);
