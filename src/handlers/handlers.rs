use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

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

    let data = Lenses::load_all_data().unwrap();

    HttpResponse::Ok().json(data)
}

#[get("/api/person/{code}")]
pub async fn find_person_from_code(web::Path(code): web::Path<String>) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");

    let person: People = people::table.filter(people::code.eq(code)).first(&conn)
        .expect("Unable to load person");

    let lenses = Lenses::belonging_to(&person)
        .load::<Lenses>(&conn)
        .expect("Error leading people");
    
    HttpResponse::Ok().json((person, lenses))
}

#[get("/person/{id}")]
pub async fn find_person() -> impl Responder {
    HttpResponse::Ok().json(NewPerson::new())
}

#[get("/lens/{id}")]
pub async fn find_lens(web::Path(id): web::Path<i32>) -> impl Responder {
    
    HttpResponse::Ok().json(Lenses::find(id).unwrap())
}





