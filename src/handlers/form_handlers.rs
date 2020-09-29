use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use tera::Context;
use serde::Deserialize;

use crate::AppData;
use crate::models::{Lens, Domain, LivedStatement, Person};

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
pub async fn handle_lenses_form_input(data: web::Data<AppData>, req: HttpRequest, form: web::Form<FormLens>) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    println!("{:?}", form);

    let mut p = Person::new(01);

    let mut lived_statements = vec!();
    let inclusivity: f64;

    if &form.response_1 != "" {
        lived_statements.push(LivedStatement::new(form.response_1.to_owned()));
    };

    if &form.response_2 != "" {
        lived_statements.push(LivedStatement::new(form.response_2.to_owned()));
    };

    if &form.response_3 != "" {
        lived_statements.push(LivedStatement::new(form.response_3.to_owned()));
    };

    inclusivity = form.inclusivity as f64 / 100.0;

    let l = Lens::new(
        form.name.to_owned(),
        form.domain.to_owned(),
        lived_statements,
        inclusivity,
    );

    p.lenses.push(l);

    println!("{:?}", p);

    HttpResponse::Found().header("Location", "/lens_form").finish()
}