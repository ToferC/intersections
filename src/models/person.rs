use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};

use super::lens::Lens;

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    uid: i64,
    code: String,
    hash: String,
    pub lenses: Vec<Lens>,
}

impl Person {
    pub fn new(uid: i64) -> Person {
        Person {
            uid: uid,
            code: generate_unique_code(),
            hash: String::from("Barking Willow Tree"),
            lenses: vec!()
        }
    }
}

fn generate_unique_code() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(9)
        .collect();

    rand_string
}