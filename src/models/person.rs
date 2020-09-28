use super::lens::Lens;

pub struct Person {
    uid: i64,
    email: String,
    lenses: Vec<Lens>,
}

impl Person {
    pub fn new(uid: i64, email: String) -> Person {
        Person {
            uid: uid,
            email: email,
            lenses: vec!()
        }
    }
}