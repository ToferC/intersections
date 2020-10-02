CREATE EXTENSION IF NOT EXISTS "uuid-ossp";



CREATE TABLE persons (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    hashcode VARCHAR(30) NOT NULL,
    date_created TIMESTAMP NOT NULL
);

CREATE TYPE domain AS ENUM ('person', 'role', 'system');

CREATE TABLE domains (
    id SERIAL PRIMARY KEY,
    domain_token domain NOT NULL
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    lens_name VARCHAR(255) NOT NULL,
    date_created TIMESTAMP NOT NULL,
    domain VARCHAR(10) NOT NULL,
    inclusivity INT NOT NULL,
    statements TEXT[],
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES persons(id) ON DELETE CASCADE
);