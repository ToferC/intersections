#[macro_use]
extern crate diesel;

use models::People;
use tera::Tera;
use actix_session::Session;
use actix_identity::Identity;
use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::{ToBigInt};

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

pub fn prepopulate_db() {

    // prepopulate database
    let test_admin = models::User::create(
        models::UserData {
            user_name: "admin".to_owned(),
            email: "admin@email.com".to_owned(),
            password: "ChumbaWumba11".to_owned(),
            role: "admin".to_owned(),
        }
    ).expect("Unable to create tets admin");

    let general_community = models::Communities::create(
        &models::NewCommunity::new(
            "Test Community".to_owned(),
            "A test community populated with dummy data".to_owned(),
            "test data in app".to_owned(),
            "admin@email.com".to_owned(),
            true,
            test_admin.id,
        )
    ).expect("Unable to create generic community");

    for _ in 0..4 {
        let _person = models::People::create(
            &models::NewPerson::new(general_community.id)
        ).expect("Unable to create new person {}");
    };

    let base_lenses = vec![
        ("father", "person", 1, 1, "tired", "not doing enough", "joyful", -0.18),
        ("manager", "role", 1, 2, "pulled many directions", "influential", "stressed", -0.25),
        ("gen x", "person", 1, 3, "experienced", "overlooked", "depended upon", 0.23),
        ("mother", "person", 2, 4, "tired", "guilty", "excluded", -0.45),
        ("white", "person", 2, 5, "normal", "", "", 0.30),
        ("black", "person", 3, 6, "ignored", "suffer microagressions", "proud", -0.30),
        ("mother", "person", 3, 4, "balanced", "responsible", "capable", 0.29),
        ("executive", "role",3, 7, "powerful", "overwhelmed", "stifled", -0.10),
        ("innovator", "role", 3, 8, "respected", "sidelined", "not recognized by system", 0.20),
        ("white", "person", 4, 5, "listened to", "persecuted by diversity iniatives", "comfortable", 0.09),
    ];

    for l in base_lenses.iter() {

        let i = l.7 as f32;
        let inclusivity = BigDecimal::new(i.to_bigint().unwrap(), 2);

        models::Lenses::create(
            &models::Lens::new(
                l.0.to_string(), 
                l.1.to_string(), 
                l.2, 
                l.3, 
                vec![l.4.to_string(), l.5.to_string(), l.6.to_string()], 
                inclusivity,
            )
        ).expect("Unable to create lens");
    }
}