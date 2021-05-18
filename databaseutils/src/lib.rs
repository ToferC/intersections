use bigdecimal::{BigDecimal, ToPrimitive};
use inflector::Inflector;
use num_bigint::{ToBigInt};
use std::{io::{stdin}, process::exit};
use std::{num::ParseIntError};
use std::collections::{BTreeMap};

use std::fs::File;
use serde_json::Value;

use libretranslate::{translate, Language};

use error_handler::error_handler::CustomError; 
use webapp::models::{self, RawExperience, Phrases};
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
        validated: true,
    };

    let user = models::User::create(user_data)?;

    println!("New user created: {:?}", &user);

    println!("End Script");
    
    Ok(user.id)
}

pub fn create_test_admin() -> Result<i32, CustomError> {
    // prepopulate database
    let mut test_admin = models::User::create(
        models::UserData {
            user_name: "admin".to_owned(),
            email: "admin@email.com".to_owned(),
            password: "ChumbaWumba11".to_owned(),
            role: "admin".to_owned(),
            validated: true,
        }
    )?;

    test_admin.validated = true;

    println!("TEST ADMIN CREATED: {:?}", &test_admin);

    Ok(test_admin.id)
}

pub async fn prepopulate_db(mode: &str) {

    // choose admin

    add_base_nodes().await;

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

            let mut test_id:i32;

            let test_communities = models::Communities::find_test_ids()
                .expect("Unable to load communities");

            if test_communities.len() == 0 as usize {

                let test_community_data = &models::NewCommunity::new(
                    "Demo Community".to_owned(),
                    "Original alpha test data for intersections. This data is a mix of dummy data, demonstration data and real people testing the platform. It is excluded from the global data set and can only be accessed as a separate community.".to_owned(),
                    "Demonstration of test data in app".to_owned(),
                    owner.email.to_owned(),
                    true,
                    owner.id,
                    true,
                );

                let test_community = models::Communities::create(
                    test_community_data
                ).expect("Unable to create demo community");

                test_id = test_community.id;

                // Generate global community placeholder
                let global_community_data = &models::NewCommunity::new(
                    "Global".to_owned(),
                    "Global community for disaggregated data".to_owned(),
                    "Holder for disaggregated community data".to_owned(),
                    owner.email.to_owned(),
                    true,
                    owner.id,
                    false,
                );

                let _global_community = models::Communities::create(
                    global_community_data
                ).expect("Unable to create generic community");


            } else {
                test_id = test_communities[0];
            };

            match mode {
                "demo" => import_demo_data(test_id).await,
                _ => generate_dummy_data(test_id).await,
            };

            println!("SUCCESS");

        },
        Err(e) => {
            println!("No administrators found in DB. Try creating an admin user first. Error {}", e);
            exit(0)
        }
    }
}

pub async fn import_demo_data(community_id: i32) {

    add_base_nodes().await;
    
    let json_path = "test_data.json";

    let file = File::open(json_path).unwrap();
    let data: Vec<Vec<serde_json::Value>> = serde_json::from_reader(file).unwrap();

    let mut community = models::Communities::find(community_id).expect("Unable to load community");

    let comm_data: models::CommunityData = serde_json::from_value(community.data).unwrap();

    let mut comm_data = comm_data.to_owned();
    let mut temp_incl_map: BTreeMap<i32, f32> = comm_data.inclusivity_map.clone();

    let mut raw_experience_vec: Vec<RawExperience> = Vec::new();

    // enter base data and prepare raw experience vecs for translation
    for e in &data {

        // create people
        let p = models::NewPerson::new(community_id);

        let person = models::People::create(&p).expect("Unable to insert person.");

        comm_data.members += 1;


        for i in e[1].as_array() { // experience Array per each person
            
            for n in i {
                let name = n[1]["node_name"].as_str().unwrap().trim().to_owned();

                let mut statements: Vec<String> = Vec::new();

                for s in n[0]["statements"].as_array().unwrap() {
                    statements.push(s.as_str().unwrap().trim().to_owned());
                };

                let mut raw_exp = RawExperience::new(
                    name.to_owned(),
                    statements,
                );

                let _result = raw_exp.generate_experience_phrases("en")
                    .await
                    .expect("Unable to generate phrases for experience.");

                let raw_num: f64 = n[0]["inclusivity"].as_str().unwrap().to_owned().parse().unwrap();

                let inclusivity = BigDecimal::new((raw_num * 1000.0)
                    .to_bigint()
                    .unwrap(), 3);

                let node_data = models::Node::new(
                    raw_exp.name_id,
                    name.to_owned(),
                    n[1]["domain_token"].as_str().unwrap().trim().to_owned(),
                );
    
                let node = models::Nodes::create(&node_data);

                let (node, _node_name) = match node {
                    Ok(n) => (n, Phrases { id: raw_exp.name_id, lang: "en".to_string(), text: name.to_owned(), machine_translation: false}),
                    Err(e) => {
                        println!("{}", e);
                        models::Nodes::find_by_slug(&name.trim().to_snake_case(), "en").expect("Unable to load node")
                    }
                };

                let l = models::Experience::new(
                    raw_exp.name_id.clone(),
                    node.domain_token.to_owned(),
                    person.id,
                    node.id, 
                    raw_exp.phrase_ids.clone(),
                    inclusivity.to_owned(),
                    node.slug,
                );

                let ex = models::Experiences::create(&l).expect("Unable to create experience");
                
                comm_data.experiences += 1;
                temp_incl_map.insert(ex.id, inclusivity.to_f32().unwrap());

                let total: f32 = temp_incl_map.values().sum();

                comm_data.mean_inclusivity = total / temp_incl_map.len() as f32;

                raw_experience_vec.push(raw_exp);
            };
        };
    };

    // Break vec into chunks and process
    for e in raw_experience_vec.chunks(10) {
        batch_translate(e.to_vec(), "en").await;
    };

    comm_data.inclusivity_map = temp_incl_map;
    community.data = serde_json::to_value(comm_data).unwrap();

    let update = models::Communities::update(&community);

    match update {
        Ok(c)=> println!("Community Updated: {}", c.tag),
        Err(e) => println!("Error:{}", e),
    }

    println!("");
}

pub async fn generate_dummy_data(community_id: i32) {
    for _ in 0..4 {
        let _person = models::People::create(
            &models::NewPerson::new(community_id)
        ).expect("Unable to create new person {}");
    };

    add_base_nodes().await;

    let base_experiences = vec![
        ("father", "person", 1, 1, "tired", "not doing enough", "joyful", -0.18),
        ("manager", "role", 1, 2, "pulled many directions", "influential", "stressed", -0.25),
        ("gen x", "person", 1, 3, "experienced", "overlooked", "depended upon", 0.23),
        ("mother", "person", 2, 4, "tired", "guilty", "excluded", -0.45),
        ("white", "person", 2, 5, "normal", "just a person", "listened to", 0.30),
        ("black", "person", 3, 6, "ignored", "suffer microagressions", "proud", -0.30),
        ("mother", "person", 3, 4, "balanced", "responsible", "capable", 0.29),
        ("executive", "role",3, 7, "powerful", "overwhelmed", "stifled", -0.10),
        ("innovator", "role", 3, 8, "respected", "sidelined", "not recognized by system", 0.20),
        ("white", "person", 4, 5, "listened to", "persecuted by diversity iniatives", "comfortable", 0.09),
    ];

    let file = File::open("test_data.json").unwrap();
    let data: Value = serde_json::from_reader(file).unwrap();

    println!("{}", &data);

    let mut raw_exp_vec = Vec::new();

    for l in base_experiences.iter() {

        let raw_exp = RawExperience::new( 
            l.0.to_string(), 
            vec![l.4.to_string(), l.5.to_string(), l.6.to_string()],
        );

        raw_exp_vec.push(raw_exp.clone());

    };

    let mut translate_strings: Vec<String> = Vec::new();
        
    for e in &raw_exp_vec {
        translate_strings.push(format!("{}.\n", &e.node_name));

        for (i, s) in e.statements.clone().iter().enumerate() {
            if s != "" {
                if i+1 == e.statements.len() {
                    translate_strings.push(format!("{}", &s));
                } else {
                    translate_strings.push(format!("{}.\n", &s));
                }
            };
        };
    };

    let source = Language::English;
    let target = Language::French;

    let input = translate_strings.concat();

    let data = translate(source, target, input)
        .await
        .unwrap();

    //let en = data.input.split(". ").into_iter();
    let fr: Vec<String> = data.output.split(".\n").map(|s| s.to_string()).collect();
    //let en = data.input.split(".\n");
    
    for (i, l) in base_experiences.iter().enumerate() {
        
        let mut exp = raw_exp_vec[i].clone();

        // 4 strings per experience in translation (0, 1, 2, 3), (4, 5, 6, 7), (8, 9, 10, 11)
        // pull node_name and 3 statement ids from each l and raw_exp_vec and fr

        for s in 0+i*4..4+i*4 {
            if s == 0+i*4 {
                // node_name
                let prep_phrase = models::InsertablePhrase::new("en", exp.node_name.to_owned(), false);
    
                // check to see if phrase already exists, return phrase if it does
                let p = Phrases::find_from_text(&prep_phrase.text, &prep_phrase.lang);

                println!("Checking to see if phrase: {} exists", &prep_phrase.text);
                let phrase = match p {
                    Ok(p) => {
                        println!("Exists");
                        p
                    },
                    Err(e) => {
                        println!("Does not exist - creating{}", e);
                        Phrases::create(&prep_phrase).expect("Unable to create phrase")
                    }
                };
    
                let trans = models::Phrases {
                    id: phrase.id,
                    lang: "fr".to_string(),
                    text: fr[s].to_lowercase().replace("/",""),
                    machine_translation: true,
                };
    
                // see if translation exists
                let p = Phrases::find_from_text(&trans.text, &trans.lang);
                
                println!("Checking to see if phrase: {} exists", &trans.text);
                let translation = match p {
                    Ok(p) => {
                        println!("Translation exists");
                        p
                    },
                    Err(e) => {
                        println!("Does not exist - creating{}", e);
                        Phrases::add_translation(trans).expect("unable to add translation")
                    }
                };

                println!("Success - node name: {} ({}) -> {} ({})", &phrase.text, phrase.id, &translation.text, translation.id);

                // update raw_experience
                exp.name_id = phrase.id;
                
            } else {

                //statement
                let prep_phrase = models::InsertablePhrase::new("en", exp.statements[s-1-i*4].clone(), false);
    
                // check to see if phrase already exists, return phrase if it does
                let p = Phrases::find_from_text(&prep_phrase.text, &prep_phrase.lang);

                println!("Checking to see if phrase: {} exists", &prep_phrase.text);
                let phrase = match p {
                    Ok(p) => {
                        println!("Exists");
                        p
                    },
                    Err(e) => {
                        println!("Does not exist - creating{}", e);
                        Phrases::create(&prep_phrase).expect("Unable to create phrase")
                    }
                };
    
                let trans = models::Phrases {
                    id: phrase.id,
                    lang: "fr".to_string(),
                    text: fr[s].to_lowercase().replace("/",""),
                    machine_translation: true,
                };
    
                // see if translation exists
                let p = Phrases::find_from_text(&trans.text, &trans.lang);
                
                println!("Checking to see if phrase: {} exists", &trans.text);
                let translation = match p {
                    Ok(p) => {
                        println!("Translation exists");
                        p
                    },
                    Err(e) => {
                        println!("Does not exist - creating{}", e);
                        Phrases::add_translation(trans).expect("unable to add translation")
                    }
                };

                // update raw_experience
                println!("Success - statement: {} ({}) -> {} ({})", &phrase.text, phrase.id, &translation.text, translation.id);

                exp.phrase_ids.push(phrase.id);
            };
        };
    
        let i = l.7 as f32;
        let inclusivity = BigDecimal::new(i.to_bigint().unwrap(), 2);
        
        models::Experiences::create(
            &models::Experience::new(
                exp.name_id, 
                l.1.to_string(), 
                l.2, 
                l.3, 
                exp.phrase_ids, 
                inclusivity,
                exp.node_name.trim().to_snake_case().to_string(),
            )
        ).expect("Unable to create experience");
    }
    println!("************ADDED DEMO EXPERIENCES*************");
}

/// Add base nodes to database for autosuggest
pub async fn add_base_nodes() {

    println!("************ADDING BASE NODES*************");

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
            ("eastern european", "person"),
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
            ("human resources officer", "role"),
            ("executive", "role"),
            ("finance officer", "role"),
            ("supervisor", "role"),
            ("psychologist", "role"),
            ("teacher", "role"),
            ("educator", "role"),
            ("developer", "role"),
            ("leader", "role"),
            ("networker", "role"),
            ("entrepreneur", "role"),
            ("policy analyst", "role"),
        ];

        let mut translate_strings: Vec<String> = Vec::new();
        
        for n in &nodes {
            translate_strings.push(format!("{}.\n", &n.0));
        };

        let source = Language::English;
        let target = Language::French;

        let input = translate_strings.concat();

        println!("Sending Translation to LibreTranslate");

        let data = translate(source, target, input)
            .await;

        match data {
            Ok(data) => {

                //let en = data.input.split(". ").into_iter();
                let fr = data.output.split(".\n");
        
                let copy = nodes.clone();
        
                for (n, f) in copy.iter().zip(fr) {
        
                    let prep_phrase = models::InsertablePhrase::new("en", n.0.to_owned(), false);
        
                    // check to see if phrase already exists, return phrase if it does
                    let p = Phrases::find_from_text(&prep_phrase.text, &prep_phrase.lang);

                    println!("Checking to see if phrase: {} exists", &prep_phrase.text);
                    let phrase = match p {
                        Ok(p) => {
                            println!("Exists");
                            p
                        },
                        Err(e) => {
                            println!("Does not exist - creating{}", e);
                            Phrases::create(&prep_phrase).expect("Unable to create phrase")
                        }
                    };
        
                    let trans = models::Phrases {
                        id: phrase.id,
                        lang: "fr".to_string(),
                        text: f.to_lowercase().replace("/",""),
                        machine_translation: true,
                    };
        
                    // see if translation exists
                    let p = Phrases::find_from_text(&trans.text, &trans.lang);
                    
                    println!("Checking to see if phrase: {} exists", &trans.text);
                    let translation = match p {
                        Ok(p) => {
                            println!("Translation exists");
                            p
                        },
                        Err(e) => {
                            println!("Does not exist - creating{}", e);
                            Phrases::add_translation(trans).expect("unable to add translation")
                        }
                    };
                    
                    let node = models::Node::new(
                        phrase.id,
                        n.0.to_lower_case().trim().to_string(),
                        n.1.to_owned(),
                    );
        
                    println!("Success: {} ({}) -> {} ({})", &phrase.text, phrase.id, &translation.text, translation.id);
                    
                    let _ = models::Nodes::create(&node);
                };
        
                println!("************ADDED BASE NODES*************");
            },
            Err(e) => {
                println!("{}", e);
            }
        };
    };
}

pub async fn batch_translate(raw_experience_vec: Vec<RawExperience>, lang: &str) {
    // send text to Libretranslate
    let mut translate_strings: Vec<String> = Vec::new();
        
    for e in &raw_experience_vec {
        translate_strings.push(format!("{}.\n", &e.node_name));

        for (i, s) in e.statements.clone().iter().enumerate() {
            if s != "" {
                if i+1 == e.statements.len() {
                    translate_strings.push(format!("{}.\n", &s));
                } else {
                    translate_strings.push(format!("{}.\n", &s));
                }
            };
        };
    };

    // Prepare translation
    let mut source = Language::English;
    let mut target = Language::French;
    
    let translate_lang = match &lang {
        &"en" => {
            "fr".to_string()
        },
        &"fr" => {
            source = Language::French;
            target = Language::English;
            "en".to_string()
        },
        _ => {
            "fr".to_string()
        },
    };

    let input = translate_strings.concat();

    // Send translation
    println!("Sending translation to Libretranslate");

    let data = translate(source, target, input)
        .await
        .unwrap();

    //let en = data.input.split(". ").into_iter();
    let fr: Vec<String> = data.output.split(".\n").map(|s| s.to_string()).collect();

    let mut row_counter: usize = 0;
    for e in raw_experience_vec.iter() {
        
        // node_name
        let trans = models::Phrases {
            id: e.name_id,
            lang: translate_lang.to_string(),
            text: fr[row_counter].to_lowercase().replace("/",""),
            machine_translation: true,
        };

        // see if translation exists
        let p = Phrases::find_from_text(&trans.text, &trans.lang);
        
        println!("Checking to see if phrase: {} exists", &trans.text);
        let translation = match p {
            Ok(p) => {
                println!("Translation exists");
                p
            },
            Err(e) => {
                println!("Does not exist - creating{}", e);
                Phrases::add_translation(trans).expect("unable to add translation")
            }
        };

        println!("Success - statement: {} ({}) -> {} ({})", &e.node_name, e.name_id, &translation.text, translation.id);

        row_counter += 1;

        // statements
        for phrase_id in e.phrase_ids.clone() {
            // node_name
            let trans = models::Phrases {
                id: phrase_id,
                lang: "fr".to_string(),
                text: fr[row_counter].to_lowercase().replace("/",""),
                machine_translation: true,
            };

            row_counter += 1;

            // see if translation exists
            let p = Phrases::find_from_text(&trans.text, &trans.lang);
            
            println!("Checking to see if phrase: {} exists", &trans.text);
            let translation = match p {
                Ok(p) => {
                    println!("Translation exists");
                    p
                },
                Err(e) => {
                    println!("Does not exist - creating{}", e);
                    Phrases::add_translation(trans).expect("unable to add translation")
                }
            };

            println!("Success - statement: {} ({}) -> {} ({})", &e.node_name, e.name_id, &translation.text, translation.id);

        };
    };
}