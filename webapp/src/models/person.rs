use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl, BelongingToDsl};

use crate::schema::{people};
use crate::models::{Lenses};
use error_handler::error_handler::CustomError;
use database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable, Clone)]
#[table_name = "people"]
pub struct NewPerson {
    pub code: String,
    pub date_created: chrono::NaiveDateTime,
    pub related_codes: Vec<String>,
    pub community_id: i32,
}

impl NewPerson {
    pub fn new() -> NewPerson {
        NewPerson {
            code: generate_unique_code(),
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            related_codes: Vec::new(),
            community_id: 0,
        }
    }

    pub fn from(person: &NewPerson) -> NewPerson {

        let now = Utc::now().naive_utc();

        NewPerson {
            code: person.code.to_owned(),
            date_created: now,
            related_codes: person.related_codes.to_owned(),
            community_id: person.community_id,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Associations, Identifiable, Clone)]
#[table_name = "people"]
pub struct People {
    pub id: i32,
    pub code: String,
    pub date_created: NaiveDateTime,
    pub related_codes: Vec<String>,
    pub community_id: i32,
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

    pub fn get_lenses(&self) -> Result<Vec<(People, Vec<Lenses>)>, CustomError> {

        let conn = database::connection()?;

        let mut people_vec: Vec<People> = Vec::new();

        let target_person = People::find(self.id).expect("Unable to load person");

        let zero_len: usize = 0;

        if &target_person.related_codes.len() > &zero_len {
            people_vec.push(target_person.clone());

            for c in &target_person.related_codes {
                people_vec.push(People::find_from_code(c).unwrap());
            }
        } else {
            people_vec.push(target_person);
        };

        // join lenses and nodes
        let lenses = Lenses::belonging_to(&people_vec)
            .load::<Lenses>(&conn).expect("Unable to load lenses");

        // group node_lenses by people
        let grouped = lenses.grouped_by(&people_vec);

        // structure result
        let result: Vec<(People, Vec<Lenses>)> = people_vec
            .into_iter()
            .zip(grouped)
            .collect();

        Ok(result)

    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(people::table.filter(people::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

pub fn generate_unique_code() -> String {
    let mut rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string.insert(6, '-');
    rand_string.insert(3, '-');

    rand_string
}