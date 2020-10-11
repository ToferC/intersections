CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    related_codes TEXT[] NOT NULL
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) UNIQUE NOT NULL,
    domain_token VARCHAR(10) NOT NULL
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(10) NOT NULL,
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