#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use tera::Tera;
use actix_session::Session;

pub mod models;
pub mod handlers;
pub mod schema;

pub struct AppData {
    pub tmpl: Tera,
}

pub fn extract_session_data(session: &Session) -> (String, String) {

    let role_data = session.get::<String>("role").expect("Unable to get role from cookie");

    let role = match role_data {
        Some(r) => r,
        None => "".to_string(),
    };

    let user_data = session.get::<String>("user_name").expect("Unable to get user_name from cookie");

    let session_user = match user_data {
        Some(u) => u,
        None => "".to_string(),
    };

    println!("{}-{}", &session_user, &role);

    (session_user, role)
}