use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl, BelongingToDsl};

use crate::schema::{communities};
use crate::models::{People, generate_unique_code};
use crate::error_handler::CustomError;
use crate::database;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable, Clone)]
#[table_name = "communities"]
pub struct NewCommunity {
    pub tag: String,
    pub date_created: chrono::NaiveDateTime,
    pub code: String,
}

impl NewCommunity {
    pub fn new(tag: String) -> NewCommunity {
        NewCommunity {
            tag: tag,
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            code: generate_unique_code(),
        }
    }

    pub fn from(community: &NewCommunity) -> NewCommunity {
        let now = Utc::now().naive_utc();

        NewCommunity {
            tag: community.tag.to_owned(),
            date_created: now,
            code: community.code.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Assocations, Identifiable, Clone)]
#[table_name = "communities"]
pub struct Communities {
    pub id: i32,
    pub tag: String,
    pub date_created: NaiveDateTime,
    pub code: String,
}

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

    pub fn find_from_code(code: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let community = communities::table.filter(communities::code.eq(code)).first(&conn)?;
        Ok(community)
    }

    pub fn delete(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let res = deisel::delete(communities::table.filter(id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

