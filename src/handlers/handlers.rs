use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};

use num_bigint::{BigInt, ToBigInt};
use bigdecimal::BigDecimal;

use crate::models::{Person, Lens, Domain};
use crate::handlers::{lens_form_handler, handle_lens_form_input,
    };

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(index);
    config.service(find_person);
    config.service(find_lens);
    config.service(api_base);
    config.service(lens_form_handler);
    config.service(handle_lens_form_input);
}

#[get("/")]
pub async fn index(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
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
        String::from("Default"),
        Domain::Person,
        vec!(),
        BigDecimal::new(r.to_bigint().unwrap(), -2),     
    ))
}





