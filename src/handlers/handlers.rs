use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Lenses, Nodes, People};
use crate::database;

use crate::schema::{nodes};


#[get("/")]
pub async fn index(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    println!("Access index");

    let node_vec = Nodes::find_all_names().expect("Unable to load nodes");

    let mut ctx = Context::new();

    ctx.insert("nodes", &node_vec);

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api")]
pub async fn api_base() -> impl Responder {

    let data = Lenses::load_api_data().unwrap();

    HttpResponse::Ok().json(data)
}

#[get("/api/person/{code}")]
pub async fn person_api(web::Path(code): web::Path<String>) -> impl Responder {
    
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





