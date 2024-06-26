use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl};
use std::collections::{BTreeMap};

use bigdecimal::{ToPrimitive};

use crate::schema::{communities, phrases};
use crate::generate_unique_code;
use crate::models::{People, Experiences, Phrases};
use error_handler::error_handler::CustomError;
use database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityData {
    pub members: i32,
    pub experiences: i32,
    pub diversity: f64,
    pub inclusivity_map: BTreeMap<i32, f32>,
    pub mean_inclusivity: f32,
    pub tags: Vec<String>,
    pub invitations: i32,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable, Clone)]
#[table_name = "communities"]
pub struct NewCommunity {
    pub tag: i32,
    pub description: i32,
    pub data_use_case: i32,
    pub contact_email: String,
    pub date_created: chrono::NaiveDateTime,
    pub open: bool,
    pub code: String,
    pub slug: String,
    pub user_id: i32,
    pub data: serde_json::Value,
    pub test: bool,
}

impl NewCommunity {
    pub fn new(tag: i32, description: i32, data_use_case: i32, contact_email: String, open: bool, slug: String, user_id: i32, test: bool) -> NewCommunity {

        let comm_data = CommunityData {
            members: 0,
            experiences: 0,
            diversity: 0.0,
            inclusivity_map: BTreeMap::new(),
            mean_inclusivity: 0.0,
            tags: Vec::new(),
            invitations: 0,
        };

        NewCommunity {
            tag,
            description,
            data_use_case,
            contact_email,
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            open,
            code: generate_unique_code(24, true),
            slug,
            user_id,
            data: serde_json::to_value(&comm_data).unwrap(),
            test,
        }
    }

    pub fn from(community: &NewCommunity) -> NewCommunity {
        let now = Utc::now().naive_utc();

        NewCommunity {
            tag: community.tag,
            description: community.description,
            data_use_case: community.data_use_case,
            contact_email: community.contact_email.to_owned(),
            date_created: now,
            open: community.open,
            code: community.code.to_owned(),
            slug: community.slug.to_owned(),
            user_id: community.user_id,
            data: community.data.to_owned(),
            test: community.test,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Identifiable, AsChangeset, Clone)]
#[table_name = "communities"]
pub struct Communities {
    pub id: i32,
    pub tag: i32,
    pub description: i32,
    pub data_use_case: i32,
    pub contact_email: String,
    pub date_created: NaiveDateTime,
    pub open: bool,
    pub code: String,
    pub slug: String,
    pub user_id: i32,
    pub data: serde_json::Value,
    pub test: bool,
}

// Database operations
impl Communities {
    pub fn create(community: &NewCommunity) -> Result<Self, CustomError> {
        let mut conn = database::connection()?;
        let community = NewCommunity::from(community);
        let community = diesel::insert_into(communities::table)
            .values(community)
            .get_result(&mut conn)?;
        Ok(community)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let mut conn = database::connection()?;
        let communities = communities::table
            .load::<Communities>(&mut conn)?;
        Ok(communities)
    }

    pub fn find_all_real() -> Result<Vec<Self>, CustomError> {
        let mut conn = database::connection()?;
        let communities = communities::table
            .filter(communities::test.eq(false))
            .load::<Communities>(&mut conn)?;
        Ok(communities)
    }

    pub fn find_test_ids() -> Result<Vec<i32>, CustomError> {
        let mut conn = database::connection()?;
        let community_ids = communities::table
            .select(communities::id)
            .filter(communities::test.eq(true))
            .load::<i32>(&mut conn)?;
        Ok(community_ids)
    }

    pub fn find_all_open() -> Result<Vec<Self>, CustomError> {
        let mut conn = database::connection()?;
        let communities = communities::table
            .filter(communities::open.eq(true))
            .load::<Communities>(&mut conn)?;
        Ok(communities)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let mut conn = database::connection()?;
        let community = communities::table.filter(communities::id.eq(id)).first(&mut conn)?;
        Ok(community)
    }

    pub fn find_by_owner_user_id(id: &i32) -> Result<Vec<Self>, CustomError> {
        let mut conn = database::connection()?;
        let community = communities::table.filter(communities::user_id.eq(id))
            .load(&mut conn)?;
        Ok(community)
    }

    pub fn find_from_code(code: &String) -> Result<Self, CustomError> {
        let mut conn = database::connection()?;
        let community = communities::table.filter(communities::code.eq(code)).first(&mut conn)?;
        Ok(community)
    }

    pub fn find_from_slug(slug: &String) -> Result<Self, CustomError> {
        let mut conn = database::connection()?;
        let community = communities::table.filter(communities::slug.eq(slug)).first(&mut conn)?;
        Ok(community)
    }

    pub fn get_tag_slugs(lang: &str) -> Result<Vec<(String, String)>, CustomError> {
        let mut conn = database::connection()?;
        let index = communities::table.inner_join(phrases::table
            .on(communities::tag.eq(phrases::id)
            .and(phrases::lang.eq(lang))))
            .select((phrases::text, communities::slug))
            .load::<(String, String)>(&mut conn)?;

        Ok(index)
    }

    pub fn update(community: &Communities) -> Result<Self, CustomError> {
        let mut conn = database::connection()?;
        let community = diesel::update(communities::table)
            .filter(communities::id.eq(community.id))
            .set(community)
            .get_result(&mut conn)?;
        Ok(community)
    }

    pub fn get_phrases(&self, lang: &str) -> BTreeMap<i32, String> {
        let mut phrases = Vec::new();

        phrases.push(self.tag);
        phrases.push(self.description);
        phrases.push(self.data_use_case);

        let p = Phrases::get_phrase_map(phrases, &lang)
            .expect("Unable to load localizations from experience");

        p
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let mut conn = database::connection()?;
        let res = diesel::delete(communities::table.filter(communities::id.eq(id))).execute(&mut conn)?;
        Ok(res)
    }

    pub fn transfer_people(source_id: i32, target_slug: &String) -> Result<Self, CustomError> {
        // transfer people from source community to target community

        let mut target = Communities::find_from_slug(target_slug).expect("Unable to load target community");
        let people: Vec<People> = People::find_from_community(source_id).expect("Unable to find people in community");

        let comm_data: CommunityData = serde_json::from_value(target.data).unwrap();

        let mut comm_data = comm_data.to_owned();

        let mut experiences: Vec<Experiences> = Vec::new();
        let mut counter: i32 = 0;

        println!("Transferring people to target");
        for person in people {
            let mut p = person.clone();
            p.community_id = target.id;
            People::update(person.id, p).expect("Unable to update person");

            // update the community based on new data
            comm_data.members += 1;
            counter += 1;

            experiences.append(&mut Experiences::find_from_people_id(person.id).unwrap());
        };

        println!("{} members transferred", counter);

        for experience in experiences {
            comm_data.experiences += 1;
            comm_data.inclusivity_map.insert(experience.id, experience.inclusivity.to_f32().unwrap());
        };

        let total: f32 = comm_data.inclusivity_map.values().sum();

        comm_data.mean_inclusivity = total / comm_data.inclusivity_map.len() as f32;
        
        target.data = serde_json::to_value(comm_data).expect("Unable to update json data");

        Communities::update(&target)
    }
}