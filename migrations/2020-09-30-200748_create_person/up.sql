
CREATE TABLE persons (
    id SERIAL PRIMARY KEY,
    code VARCHAR(9) NOT NULL,
    hashcode VARCHAR(30) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE
);

CREATE TABLE lenses (
    id SERIAL PRIMARY KEY,
    lens_name VARCHAR(255) NOT NULL,
    date_created TIMESTAMP NOT NULL default CURRENT_DATE,
    domain_token VARCHAR(10) NOT NULL,
    inclusivity NUMERIC NOT NULL,
    statements TEXT[],
    person_id INT NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES persons(id) ON DELETE CASCADE
);