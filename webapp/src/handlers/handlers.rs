use std::sync::Mutex;
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, extract_identity_data};
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Lenses, Nodes, People, Communities};
use database;

use crate::schema::{nodes};

#[get("/")]
pub async fn index(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>, 
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/survey_intro/{community_code}")]
pub async fn survey_intro(
    data: web::Data<AppData>,
    web::Path(community_code): web::Path<String>, 
    _req:HttpRequest,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
) -> impl Responder {
    println!("Access index");

    // Validate community
    let community_result = Communities::find_from_code(&community_code);
    
    match community_result {
        Ok(community) => {
            let mut ctx = Context::new();
        
            // Get session data and add to context
            let (session_user, role) = extract_identity_data(&id);
            ctx.insert("session_user", &session_user);
            ctx.insert("role", &role);

            ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
        
            // all names for form autocomplete
            let all_node_names = Nodes::find_all_names().expect("Unable to load node names");
            ctx.insert("all_node_names", &all_node_names);

            ctx.insert("community", &community);
            
            let rendered = data.tmpl.render("survey_intro.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::Found().header("Location","/").finish()
        }
    }

}

#[get("/api")]
pub async fn api_base() -> impl Responder {

    let data = Lenses::load_api_data().unwrap();

    HttpResponse::Ok().json(data)
}

#[get("/api/person/{code}")]
pub async fn person_api(
    web::Path(code): web::Path<String>,
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");

    let target_person: People =  People::find_from_code(&code).expect("Couldn't load target person");

    let mut people_vec: Vec<People> = Vec::new();

    let zero_len: usize = 0;

    if &target_person.related_codes.len() > &zero_len {
        people_vec.push(target_person.clone());

        for c in &target_person.related_codes {
            people_vec.push(People::find_from_code(c).unwrap());
        }
    } else {
        people_vec.push(target_person);
    };

    // join lenses and nodes
    let node_lenses: Vec<(Lenses, Nodes)> = Lenses::belonging_to(&people_vec)
        .inner_join(nodes::table)
        .load::<(Lenses, Nodes)>(&conn)
        .expect("Error leading people");

    // group node_lenses by people
    let grouped = node_lenses.grouped_by(&people_vec);

    // structure result
    let result: Vec<(People, Vec<(Lenses, Nodes)>)> = people_vec
        .into_iter()
        .zip(grouped)
        .collect();
    
    HttpResponse::Ok().json(result)
}

#[get("/lens/{id}")]
pub async fn find_lens(web::Path(id): web::Path<i32>) -> impl Responder {
    
    HttpResponse::Ok().json(Lenses::find(id).unwrap())
}





