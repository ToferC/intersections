#[macro_use]
extern crate diesel;

use tera::Tera;
use actix_session::Session;
use actix_identity::Identity;

use sendgrid::SGClient;
use sendgrid::{Destination, Mail};

pub mod models;
pub mod handlers;
pub mod schema;

pub struct AppData {
    pub tmpl: Tera,
    pub mail_client: SGClient,
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

pub fn extract_identity_data(id: &Identity) -> (String, String) {

    let id_data = id.identity();

    let session_user = match id_data {
        Some(u) => u,
        None => "".to_string(),
    };

    let user = models::User::find_slim_from_slug(&session_user);

    let role = match user {
        Ok(u) => u.role,
        _ => "".to_string()
    };

    println!("{}-{}", &session_user, &role);

    (session_user, role)
}

pub async fn send_email(target_address: String, email_html: &String, subject: &String, sg: SGClient) {

    let mail_info = Mail::new()
        .add_to(Destination {
            address: target_address.as_str(),
            name: "Participant",
        })
        .add_from("chris@intersectional-data.ca")
        .add_subject(subject)
        .add_html(email_html.as_str())
        .add_from_name("Chris")
        .add_header("x-system-generated".to_string(), "confirmed");

        match sg.send(mail_info) {
            Err(err) => println!("Error: {}", err),
            Ok(body) => println!("Response: {:?}", body),
        };
}