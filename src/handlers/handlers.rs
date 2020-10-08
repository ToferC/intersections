use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use serde_json::json;

use num_bigint::{ToBigInt};
use bigdecimal::BigDecimal;

use crate::models::{NewPerson, Lens, Lenses, Node, Nodes, People};
use crate::database;

use crate::schema::{people, lenses, nodes};


#[get("/")]
pub async fn index(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    println!("Access index");
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api")]
pub async fn api_base() -> impl Responder {

    let data = Lenses::load_graph_data().unwrap();

    HttpResponse::Ok().json(data)
}

#[get("/person_network_graph/{code}")]
pub async fn person_network_graph(
    web::Path(code): web::Path<String>,
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let person: People = people::table.filter(people::code.eq(code))
        .first(&conn)
        .expect("Unable to load person");
    
    let mut people_vec: Vec<People> = Vec::new();
    
    let zero_len: usize = 0;
    
    if &person.related_codes.len() > &zero_len {
        people_vec.push(person.clone());
        
        for c in &person.related_codes {
            people_vec.push(People::find_from_code(c).unwrap());
        }
    } else {
        people_vec.push(person);
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

    let j = serde_json::to_string(&result).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("full_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api/person/{code}")]
pub async fn find_person_from_code(web::Path(code): web::Path<String>) -> impl Responder {
    
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

#[get("/person/{id}")]
pub async fn find_person() -> impl Responder {
    HttpResponse::Ok().json(NewPerson::new())
}

#[get("/lens/{id}")]
pub async fn find_lens(web::Path(id): web::Path<i32>) -> impl Responder {
    
    HttpResponse::Ok().json(Lenses::find(id).unwrap())
}





