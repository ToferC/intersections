use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::{ToBigInt};
use std::{io::{stdin, stdout, copy}, process::exit};
use std::{num::ParseIntError};

use std::fs::File;
use serde_json::Value;

use error_handler::error_handler::CustomError; 
use webapp::models;
use database;

pub fn create_prod_admin() -> Result<i32, CustomError> {

    dotenv::dotenv().ok();
    database::init();

    println!("Create superuser for intersections");

    let mut user_name: String = "".to_string();
    let mut email: String = "".to_string();
    let mut hash: String = "".to_string();

    println!("Enter Username: ");
    stdin().read_line(&mut user_name).expect("Unable to read user_name");

    println!("Enter Email: ");
    stdin().read_line(&mut email).expect("Unable to read user_name");

    println!("Enter Password (minimum 12 character): ");
    stdin().read_line(&mut hash).expect("Unable to read user_name");

    let user_data: models::UserData = models::UserData {
        user_name: user_name.trim().to_string(),
        email: email.to_lowercase().trim().to_string(),
        password: hash.trim().to_string(),
        role: "admin".to_owned(),
    };

    let user = models::User::create(user_data)?;

    println!("New user created: {:?}", &user);

    println!("End Script");
    
    Ok(user.id)
}

pub fn create_test_admin() -> Result<i32, CustomError> {
    // prepopulate database
    let test_admin = models::User::create(
        models::UserData {
            user_name: "admin".to_owned(),
            email: "admin@email.com".to_owned(),
            password: "ChumbaWumba11".to_owned(),
            role: "admin".to_owned(),
        }
    )?;

    println!("TEST ADMIN CREATED: {:?}", &test_admin);

    Ok(test_admin.id)
}

pub fn prepopulate_db() {

    // choose admin

    let admins = models::User::find_admins();

    match admins {
        Ok(v) => {

            let mut admin_ids: Vec<i32> = Vec::new();

            println!("CHOOSE AN ADMIN FOR TEST COMMUNITY");
            for a in v.into_iter() {
                println!("{} - {}", a.id, a.user_name);
                admin_ids.push(a.id);
            };

            let mut response = String::new();
            stdin().read_line(&mut response).expect("Unable to read input.");

            let choice: Result<i32, ParseIntError> = response.trim().to_string().parse::<i32>();

            let target_id = match choice {
                Ok(i) => {
                    match i {
                        i if admin_ids.contains(&i) => {
                            i
                        },
                        _ => {
                            println!("You must choose an administrator");
                            99999
                        },
                    }
                },
                Err(e) => {
                    println!("{}", e);
                    99999
                }
            };

            // Exit if we don't have a real admin account
            if target_id == 99999 {
                exit(0);
            };

            let community_data = &models::NewCommunity::new(
                "Test Community".to_owned(),
                "Original alpha test data for intersections. This data is a mix of dummy data, demonstration data and real people testing the platform. It is excluded from the global data set and can only be accessed as a separate community.".to_owned(),
                "Demonstration of test data in app".to_owned(),
                "admin@email.com".to_owned(),
                true,
                target_id,
                true,
            );

            let test_community = models::Communities::create(
                community_data
            ).expect("Unable to create generic community");

            generate_dummy_data(test_community.id);

            println!("SUCCESS");

        },
        Err(e) => {
            println!("No administrators found in DB. Try creating an admin user first.");
            exit(0)
        }
    }
}

pub fn import_json_data(community_id: i32) {

}

pub fn generate_dummy_data(community_id: i32) {
    for _ in 0..4 {
        let _person = models::People::create(
            &models::NewPerson::new(community_id)
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

    let file = File::open("test_data.json").unwrap();
    let data: Value = serde_json::from_reader(file).unwrap();

    println!("{}", &data);

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