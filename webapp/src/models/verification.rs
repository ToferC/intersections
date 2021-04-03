use uuid::Uuid;
use error_handler::error_handler::CustomError;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use inflector::Inflector;

use crate::schema::users;
use database;

use shrinkwraprs::Shrinkwrap;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl};

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Identifiable, AsChangeset, Clone)]
#[table_name = "email_verification_code"]
pub struct EmailVerification {
    pub id: i32,
    pub email_address: String,
    pub activation_code: String,
    pub expires_on: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "email_verification_code"]
pub struct InsertableVerification {
    pub email_address: String,
    pub activation_code: String,
    pub expires_on: NaiveDateTime,
}

impl EmailVerification {
    pub fn create(e: &InsertableVerification) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let ev = diesel::insert_into(email_verification_code::table)
            .values(e)
            .get_result(&conn)?;
        Ok(ev)
    }
}