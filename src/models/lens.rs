use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use bigdecimal::BigDecimal;

use crate::error_handler::CustomError;
use crate::database;

use crate::models::{People, Nodes};
use crate::schema::{lenses, people, nodes};

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "lenses"]
/// Represents an intersectional lens of lived human experience.
/// Each lens will have many lenses, each of which represents one part of their
/// sum experiences.
/// Based off the Lens-Role-System framework found here: 
/// https://www.aecf.org/m/blogdoc/LensRoleSystemFramework-2013.pdf
pub struct Lens {
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the lens.
    // Expressed as "In the workplace, this lens makes me feel {adjective}."
    pub statements: Option<Vec<String>>,
    pub inclusivity: BigDecimal,
}

impl Lens {
    pub fn new(person_id: i32, node_id: i32, statements: Vec<String>, inclusivity: BigDecimal) -> Self {
        Lens {
            person_id: person_id,
            node_id: node_id, 
            date_created: chrono::Utc::now().naive_utc(),
            statements: Some(statements),
            inclusivity: inclusivity,
        }
    }

    pub fn from(lens: &Lens) -> Lens {
        Lens {
            person_id: lens.person_id,
            node_id: lens.node_id, 
            date_created: lens.date_created,
            statements: lens.statements.clone(),
            inclusivity: lens.inclusivity.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, AsChangeset, Insertable, Associations, Identifiable, Debug)]
#[belongs_to(People, foreign_key = "person_id")]
#[belongs_to(Nodes, foreign_key = "node_id")]
#[table_name = "lenses"]
pub struct Lenses {
    pub id: i32,
    pub person_id: i32,
    pub node_id: i32,
    pub date_created: chrono::NaiveDateTime,
    // A lived statement of experience based on the lens.
    // Expressed as "In the workplace, this lens makes me feel {adjective}."
    pub statements: Option<Vec<String>>,
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

    pub fn load_all_data() -> Result<(Vec<(People, Vec<Lenses>)>, Vec<Nodes>), CustomError> {
        let conn = database::connection()?;
        let people = People::find_all()?;

        let nodes = Nodes::find_all()?;

        let node_lenses = Lenses::belonging_to(&nodes)
            .load::<Lenses>(&conn)
            .expect("Error loading nodes");

        let lenses = Lenses::belonging_to(&people)
            .load::<Lenses>(&conn)
            .expect("Error leading people");

        let grouped_lenses = lenses.grouped_by(&people);

        let result: Vec<(People, Vec<Lenses>)> = people
            .into_iter()
            .zip(grouped_lenses)
            .collect();

        Ok((result, nodes))
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let lens = lenses::table.filter(lenses::id.eq(id)).first(&conn)?;
        Ok(lens)
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