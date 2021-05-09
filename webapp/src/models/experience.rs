use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use bigdecimal::BigDecimal;
use std::collections::HashMap;

use bigdecimal::{ToPrimitive};
use std::collections::BTreeMap;
use std::iter::FromIterator;

use error_handler::error_handler::CustomError;
use database;

use crate::models::{People, Nodes, Phrases};
use crate::schema::{experiences, nodes, phrases};

use super::phrase;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "experiences"]
/// Represents an intersectional experience of lived human experience.
/// Each experience will have many experiences, each of which represents one part of their
/// sum experiences.
/// Based off the experience-Role-System framework found here: 
/// https://www.aecf.org/m/blogdoc/experienceRoleSystemFramework-2013.pdf
pub struct Experience {
    pub node_name: i32,
    pub node_domain: String,
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the experience.
    // Expressed as "In the workplace, this experience makes me feel {adjective}."
    pub statements: Vec<i32>,
    pub inclusivity: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawExperience {
    // Represents raw user entered data that will be used to construct an experience and nodes
    // with translations
    pub node_name: String,
    pub statements: Vec<String>,
}

impl Experience {
    pub fn new(node_name: i32, node_domain: String, person_id: i32, node_id: i32, statements: Vec<i32>, inclusivity: BigDecimal) -> Self {
        Experience {
            node_name: node_name,
            node_domain: node_domain,
            person_id: person_id,
            node_id: node_id, 
            date_created: chrono::Utc::now().naive_utc(),
            statements: statements,
            inclusivity: inclusivity,
        }
    }

    pub fn from(experience: &Experience) -> Experience {
        Experience {
            node_name: experience.node_name,
            node_domain: experience.node_domain.clone(),
            person_id: experience.person_id,
            node_id: experience.node_id, 
            date_created: experience.date_created,
            statements: experience.statements.clone(),
            inclusivity: experience.inclusivity.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Queryable, AsChangeset, Insertable, Associations, Identifiable, Debug, Clone)]
#[belongs_to(People, foreign_key = "person_id")]
#[belongs_to(Nodes, foreign_key = "node_id")]
#[table_name = "experiences"]
pub struct Experiences {
    pub id: i32,
    pub node_name: i32,
    pub node_domain: String,
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the experience.
    // Expressed as "In the workplace, this experience makes me feel {adjective}."
    pub statements: Vec<i32>,
    pub inclusivity: BigDecimal,
}

impl Experiences {
    pub fn create(experience: &Experience) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = Experience::from(experience);
        let p = diesel::insert_into(experiences::table)
            .values(p)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let experiences = experiences::table.load::<Experiences>(&conn)?;
        Ok(experiences)
    }

    pub fn find_all_real() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;

        let real_people_ids = People::find_real_ids().expect("Unable to load real people");

        let experiences = experiences::table
            .filter(experiences::person_id.eq_any(real_people_ids))
            .load::<Experiences>(&conn)?;
            
        Ok(experiences)
    }

    pub fn find_all_node_ids() -> Result<Vec<i32>, CustomError> {
        // return vec of all node IDs (test, pre-populated and real)
        let conn = database::connection()?;

        let ids = experiences::table.select(experiences::node_id).load::<i32>(&conn)?;

        Ok(ids)
    }

    pub fn find_real_node_ids() -> Result<Vec<i32>, CustomError> {
        // return vec of user entered node IDs
        let conn = database::connection()?;

        let real_people_ids = People::find_real_ids().expect("Unable to load real people");

        let ids = experiences::table
            .select(experiences::node_id)
            .filter(experiences::person_id.eq_any(real_people_ids))
            .load::<i32>(&conn)?;

        Ok(ids)
    }

    pub fn load_api_data() -> Result<Vec<(People, Vec<(Experiences, Nodes)>)>, CustomError> {
        let conn = database::connection()?;
        let mut people = People::find_all()?;

        for mut person in people.iter_mut() {
            person.code = String::from("protected");
            person.related_codes = Vec::new();
        };

        // join experiences and nodes
        let node_experiences = Experiences::belonging_to(&people)
            .inner_join(nodes::table)
            .load::<(Experiences, Nodes)>(&conn)
            .expect("Error leading people");

        // group node_experiences by people
        let grouped_experiences = node_experiences.grouped_by(&people);

        // structure result
        let result: Vec<(People, Vec<(Experiences, Nodes)>)> = people
            .into_iter()
            .zip(grouped_experiences)
            .collect();

        Ok(result)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let experience = experiences::table.filter(experiences::id.eq(id)).first(&conn)?;
        Ok(experience)
    }

    /*
    pub fn find_from_node_id(id: i32, lang: &str) -> Result<(Vec<(Self, Phrases)>, Vec<Phrases>), CustomError> {
        let conn = database::connection()?;
        /*
        let experience_vec = experiences::table.filter(experiences::node_id.eq(id))
            .load::<Experiences>(&conn)?;
        */
        let experience_vec = experiences::table.
            inner_join(phrases::table
            .on(phrases::id.eq(experiences::node_name)
            .and(phrases::lang.eq(lang))))
            .filter(experiences::node_id.eq(id))
            .load::<(Experiences, Phrases)>(&conn)?;

        let mut phrase_ids = Vec::new();

        // get all statement localizations
        for v in experience_vec {
            phrase_ids.extend(v.0.statements.clone());
        };

        let statement_phrases: Vec<Phrases> = Phrases::get_phrases_from_ids(phrase_ids, lang)?;
        
        Ok((experience_vec, statement_phrases))
    }
    */

    pub fn find_from_node_id(id: i32, lang: &str) -> Result<Vec<(Self, Phrases)>, CustomError> {
        let conn = database::connection()?;
        /*
        let experience_vec = experiences::table.filter(experiences::node_id.eq(id))
            .load::<Experiences>(&conn)?;
        */
        let experience_vec = experiences::table.
            inner_join(phrases::table
            .on(phrases::id.eq(experiences::node_name)
            .and(phrases::lang.eq(lang))))
            .filter(experiences::node_id.eq(id))
            .load::<(Experiences, Phrases)>(&conn)?;

        Ok(experience_vec)
    }

    pub fn find_from_people_id(id: i32) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let experience_vec = experiences::table.filter(experiences::person_id.eq(id))
            .load::<Experiences>(&conn)?;
        
        Ok(experience_vec)
    }

    pub fn update(id: i32, experience: Experience) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let experience = diesel::update(experiences::table)
            .filter(experiences::id.eq(id))
            .set(experience)
            .get_result(&conn)?;
        Ok(experience)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(experiences::table.filter(experiences::id.eq(id))).execute(&conn)?;
        Ok(res)
    }

    pub fn get_phrases(&self, lang: &str) -> Vec<Phrases> {
        let mut phrase_ids = Vec::new();
        phrase_ids.push(self.node_name);

        phrase_ids.extend(&self.statements);

        let p = Phrases::get_phrases_from_ids(phrase_ids, &lang)
            .expect("Unable to load localizations from experience");

        p
    }
}

#[derive(Serialize, Debug, PartialEq, PartialOrd)]
pub struct AggregateExperience {
    pub name: String,
    pub domain: String,
    pub count: u32,
    pub mean_inclusivity: f32,
    pub frequency_distribution: Vec<(String, u32)>,
}

impl AggregateExperience {
    pub fn from(experience_vec: Vec<Experiences>, lang: &str) -> AggregateExperience {
        
        
        
        let domain = &experience_vec[0].node_domain;
        
        let mut inclusivity: f32 = 0.0;
        let mut counts = BTreeMap::new();
        
        let mut phrase_ids = Vec::new();
        
        phrase_ids.push(experience_vec[0].node_name);
        
        for e in &experience_vec {
            
            phrase_ids.extend(&e.statements);
            
            inclusivity += e.inclusivity.to_f32().expect("Unable to convert bigdecimal");
            
        };
        
        let statement_phrases = Phrases::get_phrases_from_ids(phrase_ids, &lang)
        .expect("Unable to load phrases from experience");
        
        let name = &statement_phrases[0].text;

        for s in statement_phrases.iter().skip(1) {
            *counts.entry(s.text.clone()).or_insert(0) += 1;
        };


        let mut v = Vec::from_iter(counts);
        v.sort_by(|&(_, a), &(_, b)|b.cmp(&a));

        let count = experience_vec.len() as u32;

        AggregateExperience {
            name: name.to_owned(),
            domain: domain.to_owned(),
            count: count,
            mean_inclusivity: inclusivity / count as f32,
            frequency_distribution: v,
        }
    }
}