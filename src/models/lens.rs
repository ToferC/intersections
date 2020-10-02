#[macro_use]
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

use crate::schema::lenses;
use super::person::Person;

#[derive(Serialize, Deserialize, Debug, Clone, Associations, Queryable, PartialEq)]
#[belongs_to(Person)]
#[table_name = "lenses"]
/// Represents an intersectional lens of lived human experience.
/// Each person will have many lenses, each of which represents one part of their
/// sum experiences.
/// Based off the Person-Role-System framework found here: 
/// https://www.aecf.org/m/blogdoc/PersonRoleSystemFramework-2013.pdf
pub struct Lens {
    pub person_id: i32,
    pub lens_name: String,
    pub date_created: chrono::NaiveDateTime,
    pub domain: Domain,
    // A lived statement of experience based on the lens.
    // Expressed as "In the workplace, this lens makes me feel {adjective}."
    pub statements: Vec<String>,
    pub inclusivity: i32,
}

impl Lens {
    pub fn new(name: String, domain: Domain, statements: Vec<String>, inclusivity: i32) -> Self {
        Lens {
            person_id: 1,  
            lens_name: name,
            date_created: chrono::NaiveDate::from_ymd(2020, 6, 6).and_hms(3, 3, 3),
            domain: domain,
            statements: statements,
            inclusivity: inclusivity,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DbEnum)]
/// A lived statement of experience based on the lens.
/// Expressed as "In the worksplace, this lens makes me feel {adjective}."
pub enum Domain {
    Person,
    Role,
    System,
}