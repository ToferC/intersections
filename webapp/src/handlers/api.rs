use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Experiences, Nodes, People, Phrases};
use database;

use crate::schema::{nodes};

#[get("/{lang}/api")]
pub async fn api_base(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());
    
    let rendered = data.tmpl.render("api_base.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api/experiences")]
pub async fn api_experiences() -> impl Responder {

    let data = Experiences::find_all().unwrap();

    let mut mod_data: Vec<(Experiences, Vec<Phrases>)> = Vec::new();

    for mut e in data {
        e.person_id = 999;
        e.date_created = chrono::NaiveDateTime::from_timestamp(1000000000, 0);
        let phrases = e.get_phrases("en");
        mod_data.push((e, phrases));
    };

    HttpResponse::Ok().json(mod_data)
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
    
    let mut conn = database::connection().expect("Unable to connect to db");

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
        .load::<(Experiences, Nodes)>(&mut conn)
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

#[get("/api//experience/{id}")]
pub async fn find_experience(web::Path(id): web::Path<i32>) -> impl Responder {
    
    HttpResponse::Ok().json(Experiences::find(id).unwrap())
}





