use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use bigdecimal::BigDecimal;
use num_bigint::ToBigInt;
use tera::Context;
use serde::Deserialize;

use crate::AppData;
use crate::models::{Lens, Domain, Person};

#[derive(Deserialize, Debug)]
pub struct FormLens {
    name: String,
    domain: Domain,
    response_1: String,
    response_2: String,
    response_3: String,
    inclusivity: i32,
}

#[get("/lens_form")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/lens_form")]
pub async fn handle_lens_form_input(_data: web::Data<AppData>, req: HttpRequest, form: web::Form<FormLens>) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    println!("{:?}", form);

    let mut p = Person::new();

    let mut lived_statements = vec!();
    let inclusivity: i32;

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_owned());
    };

    inclusivity = form.inclusivity;

    let l = Lens::new(
        form.name.to_owned(),
        form.domain.to_owned(),
        lived_statements,
        inclusivity,
    );

    println!("{:?} -- {:?}", l, p);

    HttpResponse::Found().header("Location", "/lens_form").finish()
}