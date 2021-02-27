use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl};

use inflector::Inflector;

use crate::schema::{communities};
use crate::models::{generate_unique_code};
use error_handler::error_handler::CustomError;
use database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable, Clone)]
#[table_name = "communities"]
pub struct NewCommunity {
    pub tag: String,
    pub description: String,
    pub data_use_case: String,
    pub contact_email: String,
    pub date_created: chrono::NaiveDateTime,
    pub open: bool,
    pub code: String,
    pub slug: String,
    pub user_id: i32,
    pub data: serde_json::Value,
}

impl NewCommunity {
    pub fn new(tag: String, description: String, data_use_case: String, contact_email: String, open: bool, user_id: i32) -> NewCommunity {
        NewCommunity {
            tag: tag.clone(),
            description,
            data_use_case,
            contact_email,
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            open,
            code: generate_unique_code(),
            slug: tag.to_snake_case(),
            user_id,
            data: serde_json::from_str(r#"{
                "members": 0,
                "mean_inclusivity": 0.0
            }"#).unwrap(),
        }
    }

    pub fn from(community: &NewCommunity) -> NewCommunity {
        let now = Utc::now().naive_utc();

        NewCommunity {
            tag: community.tag.to_owned(),
            description: community.description.to_owned(),
            data_use_case: community.data_use_case.to_owned(),
            contact_email: community.contact_email.to_owned(),
            date_created: now,
            open: community.open,
            code: community.code.to_owned(),
            slug: community.tag.to_snake_case(),
            user_id: community.user_id,
            data: community.data.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Identifiable, AsChangeset, Clone)]
#[table_name = "communities"]
pub struct Communities {
    pub id: i32,
    pub tag: String,
    pub description: String,
    pub data_use_case: String,
    pub contact_email: String,
    pub date_created: NaiveDateTime,
    pub open: bool,
    pub code: String,
    pub slug: String,
    pub user_id: i32,
    pub data: serde_json::Value,
}

// Database operations
impl Communities {
    pub fn create(community: &NewCommunity) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = NewCommunity::from(community);
        let community = diesel::insert_into(communities::table)
            .values(community)
            .get_result(&conn)?;
        Ok(community)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let communities = communities::table.load::<Communities>(&conn)?;
        Ok(communities)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = communities::table.filter(communities::id.eq(id)).first(&conn)?;
        Ok(community)
    }

    pub fn find_from_code(code: &String) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = communities::table.filter(communities::code.eq(code)).first(&conn)?;
        Ok(community)
    }

    pub fn find_from_slug(slug: &String) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = communities::table.filter(communities::slug.eq(slug)).first(&conn)?;
        Ok(community)
    }

    pub fn update(community: Communities) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = diesel::update(communities::table)
            .filter(communities::id.eq(community.id))
            .set(community)
            .get_result(&conn)?;
        Ok(community)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(communities::table.filter(communities::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

