use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use bigdecimal::BigDecimal;
use num_bigint::{ToBigInt};
use tera::Context;
use serde::Deserialize;

use crate::AppData;
use crate::models::{Lens, Person, People};

#[derive(Deserialize, Debug)]
pub struct FormLens {
    name: String,
    domain: String,
    response_1: String,
    response_2: String,
    response_3: String,
    inclusivity: BigDecimal,
}

#[get("/first_lens_form")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/first_lens_form")]
pub async fn handle_lens_form_input(_data: web::Data<AppData>, req: HttpRequest, form: web::Form<FormLens>) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    println!("{:?}", form);

    let p = Person::new();

    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), -2); 

    let l = Lens::new(
        lived_statements,
        inclusivity,
    );

    // Post person to db
    let people = People::create(&p).expect("Unable to add person to DB");

    // Check if node exists, if not create it

    // Insert lens to db

    println!("{:?} -- {:?}", l, people);

    HttpResponse::Found().header("Location", "/add_lens_form").finish()
}

#[get("/add_lens_form")]
pub async fn add_lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/add_lens_form")]
pub async fn add_handle_lens_form_input(_data: web::Data<AppData>, req: HttpRequest, form: web::Form<FormLens>) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    println!("{:?}", form);

    let p = Person::new();

    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), -2);

    let l = Lens::new(
        lived_statements,
        inclusivity,
    );

    println!("{:?} -- {:?}", l, p);

    HttpResponse::Found().header("Location", "/add_lens_form").finish()
}