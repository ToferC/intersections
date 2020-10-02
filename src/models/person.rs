use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

use super::lens::Lens;
use crate::schema::{lenses, persons};

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
}

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "persons"]
pub struct Persons {
    pub id: i32,
    pub code: String,
    pub date_created: NaiveDateTime,
}

pub fn generate_unique_code() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string
}