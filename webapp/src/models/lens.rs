use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use bigdecimal::BigDecimal;

use bigdecimal::{ToPrimitive};
use std::collections::BTreeMap;
use std::iter::FromIterator;

use error_handler::error_handler::CustomError;
use database;

use crate::models::{People, Nodes};
use crate::schema::{lenses, nodes};

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "lenses"]
/// Represents an intersectional lens of lived human experience.
/// Each lens will have many lenses, each of which represents one part of their
/// sum experiences.
/// Based off the Lens-Role-System framework found here: 
/// https://www.aecf.org/m/blogdoc/LensRoleSystemFramework-2013.pdf
pub struct Lens {
    pub node_name: String,
    pub node_domain: String,
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the lens.
    // Expressed as "In the workplace, this lens makes me feel {adjective}."
    pub statements: Vec<String>,
    pub inclusivity: BigDecimal,
}

impl Lens {
    pub fn new(node_name: String, node_domain: String, person_id: i32, node_id: i32, statements: Vec<String>, inclusivity: BigDecimal) -> Self {
        Lens {
            node_name: node_name,
            node_domain: node_domain,
            person_id: person_id,
            node_id: node_id, 
            date_created: chrono::Utc::now().naive_utc(),
            statements: statements,
            inclusivity: inclusivity,
        }
    }

    pub fn from(lens: &Lens) -> Lens {
        Lens {
            node_name: lens.node_name.clone(),
            node_domain: lens.node_domain.clone(),
            person_id: lens.person_id,
            node_id: lens.node_id, 
            date_created: lens.date_created,
            statements: lens.statements.clone(),
            inclusivity: lens.inclusivity.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Queryable, AsChangeset, Insertable, Associations, Identifiable, Debug, Clone)]
#[belongs_to(People, foreign_key = "person_id")]
#[belongs_to(Nodes, foreign_key = "node_id")]
#[table_name = "lenses"]
pub struct Lenses {
    pub id: i32,
    pub node_name: String,
    pub node_domain: String,
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the lens.
    // Expressed as "In the workplace, this lens makes me feel {adjective}."
    pub statements: Vec<String>,
    pub inclusivity: BigDecimal,
}

impl Lenses {
    pub fn create(lens: &Lens) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = Lens::from(lens);
        let p = diesel::insert_into(lenses::table)
            .values(p)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let lenses = lenses::table.load::<Lenses>(&conn)?;
        Ok(lenses)
    }

    pub fn find_all_real() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;

        let real_people_ids = People::find_real_ids().expect("Unable to load real people");

        let lenses = lenses::table
            .filter(lenses::person_id.eq_any(real_people_ids))
            .load::<Lenses>(&conn)?;
            
        Ok(lenses)
    }

    pub fn find_all_node_ids() -> Result<Vec<i32>, CustomError> {
        // return vec of all node IDs (test, pre-populated and real)
        let conn = database::connection()?;

        let ids = lenses::table.select(lenses::node_id).load::<i32>(&conn)?;

        Ok(ids)
    }

    pub fn find_real_node_ids() -> Result<Vec<i32>, CustomError> {
        // return vec of user entered node IDs
        let conn = database::connection()?;

        let real_people_ids = People::find_real_ids().expect("Unable to load real people");

        let ids = lenses::table
            .select(lenses::node_id)
            .filter(lenses::person_id.eq_any(real_people_ids))
            .load::<i32>(&conn)?;

        Ok(ids)
    }

    pub fn load_api_data() -> Result<Vec<(People, Vec<(Lenses, Nodes)>)>, CustomError> {
        let conn = database::connection()?;
        let mut people = People::find_all()?;

        for mut person in people.iter_mut() {
            person.code = String::from("protected");
            person.related_codes = Vec::new();
        };

        // join lenses and nodes
        let node_lenses = Lenses::belonging_to(&people)
            .inner_join(nodes::table)
            .load::<(Lenses, Nodes)>(&conn)
            .expect("Error leading people");

        // group node_lenses by people
        let grouped_lenses = node_lenses.grouped_by(&people);

        // structure result
        let result: Vec<(People, Vec<(Lenses, Nodes)>)> = people
            .into_iter()
            .zip(grouped_lenses)
            .collect();

        Ok(result)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let lens = lenses::table.filter(lenses::id.eq(id)).first(&conn)?;
        Ok(lens)
    }

    pub fn find_from_node_id(id: i32) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let lens_vec = lenses::table.filter(lenses::node_id.eq(id))
            .load::<Lenses>(&conn)?;
        
        Ok(lens_vec)
    }

    pub fn find_from_people_id(id: i32) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let lens_vec = lenses::table.filter(lenses::person_id.eq(id))
            .load::<Lenses>(&conn)?;
        
        Ok(lens_vec)
    }

    pub fn update(id: i32, lens: Lens) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let lens = diesel::update(lenses::table)
            .filter(lenses::id.eq(id))
            .set(lens)
            .get_result(&conn)?;
        Ok(lens)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(lenses::table.filter(lenses::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

#[derive(Serialize, Debug, PartialEq, PartialOrd)]
pub struct AggregateLens {
    pub name: String,
    pub domain: String,
    pub count: u32,
    pub mean_inclusivity: f32,
    pub frequency_distribution: Vec<(String, u32)>,
}

impl AggregateLens {
    pub fn from(lenses: Vec<Lenses>) -> AggregateLens {
        let name = &lenses[0].node_name;
        let domain = &lenses[0].node_domain;

        let mut inclusivity: f32 = 0.0;
        let mut counts = BTreeMap::new();

        for l in &lenses {
            inclusivity += l.inclusivity.to_f32().expect("Unable to convert bigdecimal");

            for s in &l.statements {
                *counts.entry(s.to_owned()).or_insert(0) += 1;
            };
        };

        let mut v = Vec::from_iter(counts);
        v.sort_by(|&(_, a), &(_, b)|b.cmp(&a));

        let count = lenses.len() as u32;

        AggregateLens {
            name: name.to_owned(),
            domain: domain.to_owned(),
            count: count,
            mean_inclusivity: inclusivity / count as f32,
            frequency_distribution: v,
        }
    }
}