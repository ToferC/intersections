use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl};

use error_handler::error_handler::CustomError;
use crate::schema::phrases;
use database;

/* Query suggestion
    experience::tables.inner_join(phrases::table
        .on(experience::node_name.eq(phrases::id)
        .and(phrases::language.eq("foo"))
        .select((all, the, columns))
*/

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[table_name = "phrases"]
pub struct InsertablePhrase {
    pub lang: String,
    pub text: String,
}

impl InsertablePhrase {
    pub fn new(lang: &str, text: String) -> Self {
        InsertablePhrase {
            lang: lang.to_string(),
            text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Associations, Insertable)]
#[table_name = "phrases"]
pub struct Phrases {
    pub id: i32,
    pub lang: String,
    pub text: String,
}

impl Phrases {
    
    pub fn create(phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn add_translation(phrase: Phrases) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn update(id: i32, phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::update(phrases::table)
            .filter(phrases::id.eq(id))
            .set(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn get_phrases_from_ids(ids: Vec<i32>, lang: &str) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table
            .filter(phrases::id.eq_any(ids)
            .and(phrases::lang.eq(lang)))
            .load::<Phrases>(&conn)?;

            Ok(phrases)
    }

    pub fn find_from_text(text: &str, lang: &str) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table
            .filter(phrases::text.eq(text)
            .and(phrases::lang.eq(lang)))
            .first(&conn)?;

            Ok(phrases)
    }

    pub fn find(id: i32, lang: &str) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let phrase = phrases::table.filter(phrases::id.eq(id)
            .and(phrases::lang.eq(lang)))
            .first(&conn)?;

        Ok(phrase)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(phrases::table.filter(phrases::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}