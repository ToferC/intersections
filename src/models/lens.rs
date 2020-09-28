use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents an intersectional lens of lived human experience.
/// Each person will have many lenses, each of which represents one part of their
/// sum experiences.
/// Based off the Person-Role-System framework found here: 
/// https://www.aecf.org/m/blogdoc/PersonRoleSystemFramework-2013.pdf
pub struct Lens {
    pub name: String,
    pub domain: Domain,
    pub statements: Vec<LivedStatement>,
    pub inclusivity: f64,
}

impl Lens {
    pub fn new(name: String, domain: Domain, statements: Vec<LivedStatement>, inclusivity: f64) -> Self {
        Lens {
            name: name,
            domain: domain,
            statements: statements,
            inclusivity: inclusivity,  
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A lived statement of experience based on the lens.
/// Expressed as "In the workplace, this lens makes me feel {adjective}."
pub struct LivedStatement {
    pub adjective: String,
}

impl Default for LivedStatement {
    fn default() -> Self {
        LivedStatement {
            adjective: String::from("Calm"),
        }
    }
}

impl LivedStatement {
    pub fn new(
        adjective: String,
    ) -> Self {
        LivedStatement {
            adjective: adjective,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A lived statement of experience based on the lens.
/// Expressed as "In the worksplace, this lens makes me feel {adjective}."
pub enum Domain {
    Person,
    Role,
    System,
}