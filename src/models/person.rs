use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

use crate::schema::{persons};
use crate::error_handler::CustomError;
use crate::database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "persons"]
pub struct Person {
    pub code: String,
    pub hashcode: String,
    pub date_created: chrono::NaiveDateTime,
}

impl Person {
    pub fn new() -> Person {
        Person {
            code: generate_unique_code(),
            hashcode: String::from("Barking Willow Tree"),
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
        }
    }

    pub fn from(person: Person) -> Person {

        let now = Utc::now().naive_utc();

        Person {
            code: person.code,
            hashcode: person.hashcode,
            date_created: now,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "persons"]
pub struct Persons {
    pub id: i32,
    pub code: String,
    pub date_created: NaiveDateTime,
}

impl Persons {
    pub fn create(person: Person) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = Person::from(person);
        let person = diesel::insert_into(persons::table)
            .values(person)
            .get_result(&conn)?;
        Ok(person)
    }
}

pub fn generate_unique_code() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string
}