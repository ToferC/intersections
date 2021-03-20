use bigdecimal::{BigDecimal};
use num_bigint::{ToBigInt};
use std::{io::{stdin}, process::exit};
use std::{num::ParseIntError};
use std::env;

use std::fs::File;
use serde_json::Value;

use error_handler::error_handler::CustomError; 
use webapp::models;
use database;

pub fn create_user(role: &str) -> Result<i32, CustomError> {

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
        role: role.to_owned(),
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

pub fn prepopulate_db(mode: &str) {

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

            // insert test community if needed

            let owner = models::User::find(target_id).expect("Unable to load user");

            let mut test_id = 0;

            let test_communities = models::Communities::find_test_ids()
                .expect("Unable to load communities");

            if test_communities.len() == 0 as usize {

                let community_data = &models::NewCommunity::new(
                    "Demo Community".to_owned(),
                    "Original alpha test data for intersections. This data is a mix of dummy data, demonstration data and real people testing the platform. It is excluded from the global data set and can only be accessed as a separate community.".to_owned(),
                    "Demonstration of test data in app".to_owned(),
                    owner.email.to_owned(),
                    true,
                    owner.id,
                    true,
                );

                let test_community = models::Communities::create(
                    community_data
                ).expect("Unable to create generic community");

                test_id = test_community.id;

            } else {
                test_id = test_communities[0];
            };

            match mode {
                "demo" => import_demo_data(test_id),
                _ => generate_dummy_data(test_id),
            };

            println!("SUCCESS");

        },
        Err(e) => {
            println!("No administrators found in DB. Try creating an admin user first. Error {}", e);
            exit(0)
        }
    }
}

pub fn import_demo_data(community_id: i32) {

    add_base_nodes();

    let environment = env::var("ENVIRONMENT");

    let e_var = match environment {
        Ok(v) => v,
        Err(_) => String::from("test"),
    };

    let mut json_path = "test_data.json";

    if e_var == String::from("production") {
        json_path = "~/test_data.json"
    };

    let file = File::open(json_path).unwrap();
    let data: Vec<Vec<serde_json::Value>> = serde_json::from_reader(file).unwrap();

    for e in &data {
        println!("PERSON");
        println!("{:?}", &e[0]); // person Object

        let p = models::NewPerson::new(community_id);

        let person = models::People::create(&p).expect("Unable to insert person.");

        println!("LENSES");
        for i in e[1].as_array() { // lens Array
            
            for n in i {
                println!("NODE");
                println!("{:?}", n[1]); // node Object
                println!("Node name: {}", n[1]["node_name"]);

                let name = n[1]["node_name"].as_str().unwrap().to_owned();

                let node_data = models::Node::new(
                    name.to_owned(),
                    n[1]["domain_token"].as_str().unwrap().to_owned(),
                );

                let node = models::Nodes::create(&node_data);

                let node = match node {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        models::Nodes::find_by_name(name.to_owned()).expect("Unable to load node")
                    }
                };

                println!("LENS");

                let mut statements: Vec<String> = Vec::new();

                for s in n[0]["statements"].as_array().unwrap() {
                    statements.push(s.as_str().unwrap().to_owned());
                };

                let raw_num: f64 = n[0]["inclusivity"].as_str().unwrap().to_owned().parse().unwrap();

                let inclusivity = BigDecimal::new((raw_num * 1000.0)
                    .to_bigint()
                    .unwrap(), 3);

                let l = models::Lens::new(
                    node.node_name.to_owned(),
                    node.domain_token.to_owned(),
                    person.id,
                    node.id, 
                    statements,
                    inclusivity,
                );

                let _ = models::Lenses::create(&l);
    
                println!("{:?}", n[0]); // lens Object
                println!("Lens statements: {}", n[0]["statements"]);
                
            };


        };
        

        println!("");
    };

}

pub fn generate_dummy_data(community_id: i32) {
    for _ in 0..4 {
        let _person = models::People::create(
            &models::NewPerson::new(community_id)
        ).expect("Unable to create new person {}");
    };

    add_base_nodes();

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

/// Add base nodes to database for autosuggest
pub fn add_base_nodes() {

    let current_nodes = models::Nodes::find_all()
        .expect("Unable to load nodes")
        .len();

    if current_nodes == 0 {

        let nodes = vec![
            ("father", "person"),
            ("manager", "role"),
            ("gen x", "person"),
            ("mother", "person"),
            ("white", "person"),
            ("black", "person"),
            ("executive", "role"),
            ("innovator", "role"),
            ("white", "person"),
            ("caucasian", "person"),
            ("black", "person"),
            ("african american", "person"),
            ("african", "person"),
            ("african canadian", "person"),
            ("indigenous", "person"),
            ("easter european", "person"),
            ("western european", "person"),
            ("mixed", "person"),
            ("latino", "person"),
            ("hispanic", "person"),
            ("metis", "person"),
            ("inuit", "person"),
            ("native", "person"),
            ("asian", "person"),
            ("japanese", "person"),
            ("korean", "person"),
            ("chinese", "person"),
            ("cambodian", "person"),
            ("arab", "person"),
            ("jewish", "person"),
            ("irish", "person"),
            ("han chinese", "person"),
            ("french", "person"),
            ("italian", "person"),
            ("russian", "person"),
            ("dutch", "person"),
            ("swedish", "person"),
            ("greek", "person"),
            ("carribean", "person"),
            ("parent", "person"),
            ("grandparent", "person"),
            ("caregiver to parents", "person"),
            ("caregiver to children", "person"),
            ("caregiver to family", "person"),
            ("volunteer", "person"),
            ("citizen", "person"),
            ("landed immigrant", "person"),
            ("work permit", "person"),
            ("study permit", "person"),
            ("undocumented", "person"),
            ("able-bodied", "person"),
            ("some physical disability", "person"),
            ("significant physical disability", "person"),
            ("vision impairment", "person"),
            ("hearing impairment", "person"),
            ("speech impairment", "person"),
            ("slim body type", "person"),
            ("average body type", "person"),
            ("large body type", "person"),
            ("property owner", "person"),
            ("renter", "person"),
            ("shelter user", "person"),
            ("homeless", "person"),
            ("neuro-typical", "person"),
            ("some neuro-divergent", "person"),
            ("significant neuro-divergent", "person"),
            ("anxiety disorder", "person"),
            ("mood disorder", "person"),
            ("psychotic disorder", "person"),
            ("eating disorder", "person"),
            ("impulse control", "person"),
            ("addiction disorder", "person"),
            ("personality disorder", "person"),
            ("obsessive-compulsive disorder", "person"),
            ("post-traumatic stress disorder", "person"),
            ("stress response syndrome", "person"),
            ("dissociative disorder", "person"),
            ("factitious disorder", "person"),
            ("sexual or gender disorder", "person"),
            ("somatic symptom disorder", "person"),
            ("tic disorder", "person"),
            ("struggling financial situation", "person"),
            ("comfortable financial situation", "person"),
            ("surplus financial situation", "person"),
            ("low socioeconomic upbringing", "person"),
            ("middle socioeconomic upbringing", "person"),
            ("upper socioeconomic upbringing", "person"),
            ("primary school education", "person"),
            ("high school education", "person"),
            ("trade school education", "person"),
            ("college education", "person"),
            ("university education", "person"),
            ("masters level education", "person"),
            ("doctorate level education", "person"),
            ("anglophone", "person"),
            ("francophone", "person"),
            ("unilingual", "person"),
            ("bilingual", "person"),
            ("non-native english speaker", "person"),
            ("non-native french speaker", "person"),
            ("cisgender", "person"),
            ("transgender", "person"),
            ("transexual", "person"),
            ("gender fluid", "person"),
            ("non-binary", "person"),
            ("genderqueer", "person"),
            ("two spirit", "person"),
            ("male", "person"),
            ("female", "person"),
            ("heterosexual", "person"),
            ("homosexual", "person"),
            ("lesbian", "person"),
            ("gay", "person"),
            ("bisexual", "person"),
            ("pansexual", "person"),
            ("bicurious", "person"),
            ("questioning", "person"),
            ("millenial", "person"),
            ("gen y", "person"),
            ("boomer", "person"),
            ("young", "person"),
            ("middle-aged", "person"),
            ("old", "person"),
            ("nearing retirement", "person"),
            ("analyst", "role"),
            ("program officer", "role"),
            ("operations", "role"),
            ("communications", "role"),
            ("law enforcement", "role"),
            ("correctional officer", "role"),
            ("marketing professional", "role"),
            ("scientist", "role"),
            ("researcher", "role"),
            ("security", "role"),
            ("computer scientist", "role"),
            ("lawyer", "role"),
            ("auditor", "role"),
            ("procurement officer", "role"),
            ("hr officer", "role"),
            ("executive", "role"),
            ("finance officer", "role"),
            ("supervisor", "role"),
            ("psychologist", "role"),
            ("teacher", "role"),
            ("educator", "role"),
            ("it officer", "role"),
            ("leader", "role"),
            ("networker", "role"),
            ("entrepreneur", "role"),
            ("policy analyst", "role"),
            ];
            
            for n in nodes {
                let node = models::Node::new(
                    n.0.to_owned(),
                    n.1.to_owned(),
                );
                
                let _ = models::Nodes::create(&node);
            };
        };
    }