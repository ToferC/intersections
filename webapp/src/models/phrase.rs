use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

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

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(phrases::table.filter(phrases::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}
