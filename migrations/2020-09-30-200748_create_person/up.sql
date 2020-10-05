CREATE TABLE persons (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    hash_code VARCHAR(30) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    node_name VARCHAR(32) UNIQUE NOT NULL,
    domain_token VARCHAR(10) NOT NULL
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    inclusivity NUMERIC NOT NULL,
    statements TEXT[],
    node_id INT NOT NULL,
    FOREIGN KEY(node_id)
        REFERENCES nodes(id) on DELETE CASCADE,
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES persons(id) ON DELETE CASCADE
);