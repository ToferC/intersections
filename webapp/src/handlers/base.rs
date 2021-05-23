use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Experiences, Nodes, People, Phrases};
use database;

use crate::schema::{nodes};

#[get("/")]
pub async fn raw_index(
    _data: web::Data<AppData>,
    _req:HttpRequest,
) -> impl Responder {

    // Redirect if somone is getting the index with no language param
    return HttpResponse::Found().header("Location", "/en").finish()
}

#[get("/{lang}")]
pub async fn index(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());
    
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/about")]
pub async fn about(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let rendered = data.tmpl.render("about.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api")]
pub async fn api_base() -> impl Responder {

    let data = Experiences::load_api_data().unwrap();

    HttpResponse::Ok().json(data)
}

#[get("/api/phrases")]
pub async fn api_phrases() -> impl Responder {

    let data = Phrases::find_all().unwrap();

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

    // join experiences and nodes
    let node_experiences: Vec<(Experiences, Nodes)> = Experiences::belonging_to(&people_vec)
        .inner_join(nodes::table)
        .load::<(Experiences, Nodes)>(&conn)
        .expect("Error leading people");

    // group node_experiences by people
    let grouped = node_experiences.grouped_by(&people_vec);

    // structure result
    let result: Vec<(People, Vec<(Experiences, Nodes)>)> = people_vec
        .into_iter()
        .zip(grouped)
        .collect();
    
    HttpResponse::Ok().json(result)
}

#[get("/experience/{id}")]
pub async fn find_experience(web::Path(id): web::Path<i32>) -> impl Responder {
    
    HttpResponse::Ok().json(Experiences::find(id).unwrap())
}





