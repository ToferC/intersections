use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};

use num_bigint::{ToBigInt};
use bigdecimal::BigDecimal;

use crate::models::{Person, Lens};


#[get("/")]
pub async fn index(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    println!("Access index");
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/api")]
pub async fn api_base() -> impl Responder {
    HttpResponse::Ok().body("Placeholder for API for Government of Canada payscales")
}

#[get("/person/{id}")]
pub async fn find_person() -> impl Responder {
    HttpResponse::Ok().json(Person::new())
}

#[get("/lens/{id}")]
pub async fn find_lens() -> impl Responder {

    let r: i64 = 88;
    
    HttpResponse::Ok().json(Lens::new(
        vec!(),
        BigDecimal::new(r.to_bigint().unwrap(), -2),     
    ))
}





