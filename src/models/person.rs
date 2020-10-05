use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use diesel::prelude::*;

use crate::schema::{people};
use crate::error_handler::CustomError;
use crate::database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "people"]
pub struct Person {
    pub code: String,
    pub hash_code: String,
    pub date_created: chrono::NaiveDateTime,
}

impl Person {
    pub fn new() -> Person {
        Person {
            code: generate_unique_code(),
            hash_code: String::from("Barking Willow Tree"),
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
        }
    }

    pub fn from(person: &Person) -> Person {

        let now = Utc::now().naive_utc();

        Person {
            code: person.code.to_owned(),
            hash_code: person.hash_code.to_owned(),
            date_created: now,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "people"]
pub struct People {
    pub id: i32,
    pub code: String,
    pub hash_code: String,
    pub date_created: NaiveDateTime,
}

impl People {
    pub fn create(person: &Person) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let p = Person::from(person);
        let p = diesel::insert_into(people::table)
            .values(p)
            .get_results(&conn)?;
        Ok(p)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let people = people::table.load::<People>(&conn)?;
        Ok(people)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = people::table.filter(people::id.eq(id)).first(&conn)?;
        Ok(person)
    }

    pub fn update(id: i32, person: Person) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let person = diesel::update(people::table)
            .filter(people::id.eq(id))
            .set(person)
            .get_results(&conn)?;
        Ok(person)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(people::table.filter(people::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

pub fn generate_unique_code() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string
}