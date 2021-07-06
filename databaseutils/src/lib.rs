use bigdecimal::{BigDecimal, ToPrimitive};
use inflector::Inflector;
use num_bigint::{ToBigInt};
use std::{io::{stdin}, process::exit};
use std::{num::ParseIntError};
use std::collections::{BTreeMap};
use std::sync::Arc;
use tokio::spawn;

use std::fs::File;
use serde_json::Value;

use deepl_api::{TranslatableTextList, DeepL};

use error_handler::error_handler::CustomError; 
use webapp::models::{self, Phrases, RawExperience, translate_experience_phrases, generate_phrases, translate_phrases};
use database;

pub fn create_user(organizational_role: &str) -> Result<i32, CustomError> {

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
        role: organizational_role.to_owned(),
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

            let test_id:i32;

            let test_communities = models::Communities::find_test_ids()
                .expect("Unable to load communities");

            if test_communities.len() == 0 as usize {

                let mut texts = Vec::new();

                texts.push("Demo Community".trim().to_owned());
                texts.push("Original alpha test data for intersections. This data is a mix of dummy data, demonstration data and real people testing the platform. It is excluded from the global data set and can only be accessed as a separate community.".to_lowercase().trim().to_owned());
                texts.push("Demonstration of test data in app".to_lowercase().trim().to_owned());

                let phrases = generate_phrases(texts, "en")
                    .await
                    .expect("Unable to generate phrases");
                
                let test_community_data = &models::NewCommunity::new(
                    phrases[0].0,
                    phrases[1].0,
                    phrases[2].0,
                    owner.email.to_owned(),
                    true,
                    phrases[0].1.to_snake_case(),
                    owner.id,
                    true,
                );

                let tm = Arc::new(phrases.clone());
                let l = Arc::new("en".to_string());

                let _translate = tokio::spawn(translate_phrases(tm, l));
                
                let test_community = models::Communities::create(
                    test_community_data
                ).expect("Unable to create demo community");

                test_id = test_community.id;

                // Generate global community placeholder
                let mut g_texts = Vec::new();

                g_texts.push("Global".to_owned());
                g_texts.push("Global community for disaggregated data".to_owned());
                g_texts.push("Holder for disaggregated community data".to_owned());

                let g_phrases = generate_phrases(g_texts, "en")
                    .await
                    .expect("Unable to generate phrases");

                let global_community_data = &models::NewCommunity::new(
                    g_phrases[0].0,
                    g_phrases[1].0,
                    g_phrases[2].0,
                    owner.email.to_owned(),
                    true,
                    g_phrases[0].1.to_snake_case(),
                    owner.id,
                    true,
                );

                let tm = Arc::new(g_phrases.clone());
                let l = Arc::new("en".to_string());

                let _translate = tokio::spawn(translate_phrases(tm, l));

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

        let mut person = models::People::create(&p).expect("Unable to insert person.");
        person.experiences = 0;

        comm_data.members += 1;


        for i in e[1].as_array() { // experience Array per each person
            
            for n in i {
                // n[0] == experience
                // n[1] == node
                let name = n[0]["node_name"].as_str().unwrap().trim().to_lower_case().to_owned();

                let mut statements: Vec<String> = Vec::new();

                for s in n[0]["statements"].as_array().unwrap() {
                    statements.push(s.as_str().unwrap().trim().to_owned());
                };

                let mut raw_exp = RawExperience::new(
                    name.to_owned(),
                    statements,
                );

                // generate phrases for experience
                let _result = raw_exp.generate_experience_phrases("en")
                    .await
                    .expect("Unable to generate phrases for experience.");

                let raw_num: f64 = n[0]["inclusivity"].as_str().unwrap().to_owned().parse().unwrap();

                let inclusivity = BigDecimal::new((raw_num * 1000.0)
                    .to_bigint()
                    .unwrap(), 3);

                let node_data = models::Node::new(
                    raw_exp.name_id,
                    raw_exp.node_name.to_owned(),
                    n[1]["domain_token"].as_str().unwrap().trim().to_owned(),
                );
    
                let node = models::Nodes::create(&node_data);

                let node = match node {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        models::Nodes::find_by_slug(&name.trim().to_snake_case()).expect("Unable to find node by slug")
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

                person.experiences += 1;
                
                comm_data.experiences += 1;
                temp_incl_map.insert(ex.id, inclusivity.to_f32().unwrap());

                let total: f32 = temp_incl_map.values().sum();

                comm_data.mean_inclusivity = total / temp_incl_map.len() as f32;

                raw_experience_vec.push(raw_exp);
            };
        };
    };

    // Machine translation for experiences
    for e in raw_experience_vec {
        // translate user strings and map to existing phrases
        // async translation thread
        let copy_raw_exp = e.clone();
        let c = Arc::new(copy_raw_exp);
        
        let l = Arc::new("en".to_string());

        println!("Sending experience to translation");
        let _translations = spawn(
            translate_experience_phrases(c, l));
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
        ("father", "relationship_caregiving", 1, 1, "tired", "not doing enough", "joyful", -0.18),
        ("manager", "organizational_role", 1, 2, "pulled many directions", "influential", "stressed", -0.25),
        ("gen x", "age", 1, 3, "experienced", "overlooked", "depended upon", 0.23),
        ("mother", "relationship_caregiving", 2, 4, "tired", "guilty", "excluded", -0.45),
        ("white", "race_culture", 2, 5, "normal", "just a person", "listened to", 0.30),
        ("black", "race_culture", 3, 6, "ignored", "suffer microagressions", "proud", -0.30),
        ("mother", "relationship_caregiving", 3, 4, "balanced", "responsible", "capable", 0.29),
        ("executive", "organizational_role",3, 7, "powerful", "overwhelmed", "stifled", -0.10),
        ("innovator", "organizational_role", 3, 8, "respected", "sidelined", "not recognized by system", 0.20),
        ("white", "race_culture", 4, 5, "listened to", "persecuted by diversity iniatives", "comfortable", 0.09),
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

    let key = match std::env::var("DEEPL_API_KEY") {
        Ok(val) if val.len() > 0 => val,
        _ => {
            eprintln!("Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.");
            std::process::exit(1);
        }
    };
    
    // send text to Deepl
    let mut translate_strings: Vec<String> = Vec::new();
        
    for e in &raw_exp_vec {
        translate_strings.push(format!("{}.\n", &e.node_name));

        for s in e.statements.clone().iter() {
            if s != "" {
                translate_strings.push(s.clone());
            };
        };
    };

    let deepl = DeepL::new(key); 

    let source = "EN".to_string();
    let target = "FR".to_string();

    println!("Sending Translation to DeepL");

    // Translate Text
    let texts = TranslatableTextList {
        source_language: Some(source),
        target_language: target,
        texts: translate_strings,
    };

    let data = deepl.translate(None, texts).await.unwrap();
    
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
                    text: data[s].text.trim().to_lowercase().replace("/",""),
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
                    text: data[s].text.to_lowercase().replace("/",""),
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

        // update based on new domains
        let nodes = vec![
            ("father", "relationship_caregiving"),
            ("manager", "organizational_role"),
            ("gen x", "age"),
            ("mother", "relationship_caregiving"),
            ("white", "race_culture"),
            ("black", "race_culture"),
            ("executive", "organizational_role"),
            ("innovator", "organizational_role"),
            ("caucasian", "race_culture"),
            ("african american", "race_culture"),
            ("african", "race_culture"),
            ("african canadian", "race_culture"),
            ("indigenous", "race_culture"),
            ("eastern european", "race_culture"),
            ("western european", "race_culture"),
            ("mixed", "race_culture"),
            ("latino", "race_culture"),
            ("hispanic", "race_culture"),
            ("metis", "race_culture"),
            ("inuit", "race_culture"),
            ("native", "race_culture"),
            ("asian", "race_culture"),
            ("japanese", "race_culture"),
            ("korean", "race_culture"),
            ("chinese", "race_culture"),
            ("cambodian", "race_culture"),
            ("arab", "race_culture"),
            ("jewish", "race_culture"),
            ("irish", "race_culture"),
            ("han chinese", "race_culture"),
            ("french", "race_culture"),
            ("italian", "race_culture"),
            ("russian", "race_culture"),
            ("dutch", "race_culture"),
            ("swedish", "race_culture"),
            ("greek", "race_culture"),
            ("carribean", "race_culture"),
            ("parent", "relationship_caregiving"),
            ("grandparent", "relationship_caregiving"),
            ("caregiver to parents", "relationship_caregiving"),
            ("caregiver to children", "relationship_caregiving"),
            ("caregiver to family", "relationship_caregiving"),
            ("volunteer", "other"),
            ("citizen", "other"),
            ("landed immigrant", "other"),
            ("work permit", "other"),
            ("study permit", "other"),
            ("undocumented", "other"),
            ("able-bodied", "ability_disability"),
            ("some physical disability", "ability_disability"),
            ("significant physical disability", "ability_disability"),
            ("vision impairment", "ability_disability"),
            ("hearing impairment", "ability_disability"),
            ("speech impairment", "ability_disability"),
            ("slim body type", "body_image"),
            ("average body type", "body_image"),
            ("large body type", "body_image"),
            ("property owner", "other"),
            ("renter", "other"),
            ("shelter user", "other"),
            ("homeless", "other"),
            ("neuro-typical", "mental_health"),
            ("some neuro-divergent", "mental_health"),
            ("significant neuro-divergent", "mental_health"),
            ("anxiety disorder", "mental_health"),
            ("mood disorder", "mental_health"),
            ("psychotic disorder", "mental_health"),
            ("eating disorder", "mental_health"),
            ("impulse control", "mental_health"),
            ("addiction disorder", "mental_health"),
            ("personality disorder", "mental_health"),
            ("obsessive-compulsive disorder", "mental_health"),
            ("post-traumatic stress disorder", "mental_health"),
            ("stress response syndrome", "mental_health"),
            ("dissociative disorder", "mental_health"),
            ("factitious disorder", "mental_health"),
            ("sexual or gender disorder", "mental_health"),
            ("somatic symptom disorder", "mental_health"),
            ("tic disorder", "mental_health"),
            ("struggling financial situation", "socio_economic"),
            ("comfortable financial situation", "socio_economic"),
            ("surplus financial situation", "socio_economic"),
            ("low socioeconomic upbringing", "socio_economic"),
            ("middle socioeconomic upbringing", "socio_economic"),
            ("upper socioeconomic upbringing", "socio_economic"),
            ("primary school education", "education"),
            ("high school education", "education"),
            ("trade school education", "education"),
            ("college education", "education"),
            ("university education", "education"),
            ("masters level education", "education"),
            ("doctorate level education", "education"),
            ("anglophone", "language"),
            ("francophone", "language"),
            ("unilingual", "language"),
            ("bilingual", "language"),
            ("non-native english speaker", "language"),
            ("non-native french speaker", "language"),
            ("cisgender", "gender"),
            ("he/him", "gender"),
            ("she/her", "gender"),
            ("they/them", "gender"),
            ("transgender", "gender"),
            ("transexual", "gender"),
            ("gender fluid", "gender"),
            ("non-binary", "gender"),
            ("genderqueer", "gender"),
            ("two spirit", "gender"),
            ("male", "gender"),
            ("female", "gender"),
            ("heterosexual", "sexuality"),
            ("homosexual", "sexuality"),
            ("lesbian", "sexuality"),
            ("gay", "sexuality"),
            ("bisexual", "sexuality"),
            ("pansexual", "sexuality"),
            ("bicurious", "sexuality"),
            ("questioning", "sexuality"),
            ("millenial", "person"),
            ("gen y", "age"),
            ("boomer", "age"),
            ("young", "age"),
            ("middle-aged", "age"),
            ("old", "age"),
            ("nearing retirement", "age"),
            ("analyst", "organizational_role"),
            ("program officer", "organizational_role"),
            ("operations", "organizational_role"),
            ("communications", "organizational_role"),
            ("law enforcement", "organizational_role"),
            ("correctional officer", "organizational_role"),
            ("marketing professional", "organizational_role"),
            ("scientist", "organizational_role"),
            ("researcher", "organizational_role"),
            ("security", "organizational_role"),
            ("computer scientist", "organizational_role"),
            ("lawyer", "organizational_role"),
            ("auditor", "organizational_role"),
            ("procurement officer", "organizational_role"),
            ("human resources officer", "organizational_role"),
            ("executive", "organizational_role"),
            ("finance officer", "organizational_role"),
            ("supervisor", "organizational_role"),
            ("psychologist", "organizational_role"),
            ("teacher", "organizational_role"),
            ("educator", "organizational_role"),
            ("developer", "organizational_role"),
            ("leader", "organizational_role"),
            ("networker", "organizational_role"),
            ("entrepreneur", "organizational_role"),
            ("policy analyst", "organizational_role"),
        ];

        let key = match std::env::var("DEEPL_API_KEY") {
            Ok(val) if val.len() > 0 => val,
            _ => {
                eprintln!("Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.");
                std::process::exit(1);
            }
        };
    
        let deepl = DeepL::new(key); 

        let source = "EN".to_string();
        let target = "FR".to_string();

        let mut translate_strings: Vec<String> = Vec::new();
        
        for n in &nodes {
            translate_strings.push(n.0.to_owned());
        };

        println!("Sending Translation to DeepL");

        // Translate Text
        let texts = TranslatableTextList {
            source_language: Some(source),
            target_language: target,
            texts: translate_strings,
        };

        let data = deepl.translate(None, texts).await;

        match data {
            Ok(translated) => {
        
                let copy = nodes.clone();
        
                for (n, f) in copy.iter().zip(translated) {
        
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
                        text: f.text.to_lowercase().replace("/",""),
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
    
    let key = match std::env::var("DEEPL_API_KEY") {
        Ok(val) if val.len() > 0 => val,
        _ => {
            eprintln!("Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.");
            std::process::exit(1);
        }
    };
    
    // send text to Deepl
    let mut translate_strings: Vec<String> = Vec::new();
        
    for e in &raw_experience_vec {
        translate_strings.push(e.node_name.clone());

        for s in e.statements.clone().iter() {
            if s != "" {
                    translate_strings.push(s.clone());
            };
        };
    };

    let deepl = DeepL::new(key); 

    let mut source = "EN".to_string();
    let mut target = "FR".to_string();

    let lang = &*lang.clone();
    
    let translate_lang = match lang {
        "en" => {
            "fr".to_string()
        },
        "fr" => {
            source = "FR".to_string();
            target = "EN".to_string();
            "en".to_string()
        },
        _ => {
            "fr".to_string()
        },
    };

    println!("Sending Translation to DeepL");

    // Translate Text
    let texts = TranslatableTextList {
        source_language: Some(source),
        target_language: target,
        texts: translate_strings,
    };

    let data = deepl.translate(None, texts).await.unwrap();

    let mut row_counter: usize = 0;

    for e in raw_experience_vec.iter() {
        
        // node_name
        let trans = models::Phrases {
            id: e.name_id,
            lang: translate_lang.to_string(),
            text: data[row_counter].text.to_lowercase().replace("/",""),
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
                text: data[row_counter].text.to_lowercase().replace("/",""),
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