use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;

use crate::schema::{people};
use crate::error_handler::CustomError;
use crate::database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable, Clone)]
#[table_name = "people"]
pub struct NewPerson {
    pub code: String,
    pub date_created: chrono::NaiveDateTime,
    pub related_codes: Vec<String>
}

impl NewPerson {
    pub fn new() -> NewPerson {
        NewPerson {
            code: generate_unique_code(),
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            related_codes: Vec::new(),
        }
    }

    pub fn from(person: &NewPerson) -> NewPerson {

        let now = Utc::now().naive_utc();

        NewPerson {
            code: person.code.to_owned(),
            date_created: now,
            related_codes: person.related_codes.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Associations, Identifiable)]
#[table_name = "people"]
pub struct People {
    pub id: i32,
    pub code: String,
    pub date_created: NaiveDateTime,
    pub related_codes: Vec<String>
}

impl People {
    pub fn create(person: &NewPerson) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = NewPerson::from(person);
        let person = diesel::insert_into(people::table)
            .values(person)
            .get_result(&conn)?;
        Ok(person)
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

    pub fn find_from_code(code: &String) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = people::table.filter(people::code.eq(code)).first(&conn)?;
        Ok(person)
    }

    pub fn update(id: i32, person: NewPerson) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = diesel::update(people::table)
            .filter(people::id.eq(id))
            .set(person)
            .get_result(&conn)?;
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