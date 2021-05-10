use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::dsl::any;
use diesel::{QueryDsl, BelongingToDsl};

use crate::schema::{people};
use crate::models::{Experiences, Communities, Phrases};
use crate::generate_unique_code;
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
    pub fn new(community_id: i32) -> NewPerson {
        NewPerson {
            code: generate_unique_code(24, true),
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            related_codes: Vec::new(),
            community_id,
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

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Associations, Identifiable, Clone, AsChangeset)]
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

    pub fn detailed_create(person: &People) -> Result<Self, CustomError> {
        let conn = database::connection()?;
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

    pub fn find_all_real() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;

        let test_commmunity_ids = Communities::find_test_ids().expect("Unable to load communities");
        
        let people = people::table
            .filter(people::community_id.ne(any(test_commmunity_ids)))
            .load::<People>(&conn)?;
        Ok(people)
    }

    pub fn find_real_ids() -> Result<Vec<i32>, CustomError> {
        let conn = database::connection()?;

        let test_commmunity_ids = Communities::find_test_ids().expect("Unable to load communities");
        
        let people_ids = people::table
            .select(people::id)
            .filter(people::community_id.ne(any(test_commmunity_ids)))
            .load::<i32>(&conn)?;

        Ok(people_ids)
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

    pub fn find_from_community(community_id: i32) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;

        let people = people::table
            .filter(people::community_id.eq(community_id))
            .load::<People>(&conn)?;

        Ok(people)
    }

    pub fn find_ids_from_community(community_id: i32) -> Result<Vec<i32>, CustomError> {
        let conn = database::connection()?;

        let people_ids = people::table
            .select(people::id)
            .filter(people::community_id.eq(community_id))
            .load::<i32>(&conn)?;

        Ok(people_ids)
    }

    pub fn update(id: i32, person: People) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let person = diesel::update(people::table)
            .filter(people::id.eq(id))
            .set(person)
            .get_result(&conn)?;
        Ok(person)
    }

    pub fn get_experiences(&self, related: bool, lang: &str) -> Result<Vec<(People, Vec<(Experiences, Vec<Phrases>)>)>, CustomError> {

        let conn = database::connection()?;

        let mut people_vec: Vec<People> = Vec::new();

        let target_person = People::find(self.id).expect("Unable to load person");

        let zero_len: usize = 0;

        if related && &target_person.related_codes.len() > &zero_len {
            // we want related codes and some exist
            people_vec.push(target_person.clone());

            for c in &target_person.related_codes {
                people_vec.push(People::find_from_code(c).unwrap());
            }
        } else {
            // only add the original person profile
            people_vec.push(target_person);
        };

        // join experiences and nodes
        let experiences = Experiences::belonging_to(&people_vec)
            .load::<Experiences>(&conn).expect("Unable to load experiences");

        let mut xp = Vec::new();

        for x in experiences {
            let p = x.get_phrases(&lang);
            xp.push((x, p));
        }

        // group node_experiences by people
        let grouped = xp.grouped_by(&people_vec);

        // structure result
        let result: Vec<(People, Vec<(Experiences, Vec<Phrases>)>)> = people_vec
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