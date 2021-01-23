use std::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use tera::Context;
use serde::{Deserialize, Serialize};

use crate::AppData;
use crate::models::{User, SlimUser, LoggedUser};
use crate::error_handler::CustomError;

#[get("/log_in")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    let rendered = data.tmpl.render("log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/log_in")]
pub async fn handle_lens_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };

    HttpResponse::Found().header("Location", format!("/add_lens_form/{}", new_person.code)).finish()
}

#[get("/register")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    let rendered = data.tmpl.render("log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/register")]
pub async fn handle_lens_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };

    HttpResponse::Found().header("Location", "/").finish()
}

#[post("/log_out")]
pub async fn handle_lens_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };

    HttpResponse::Found().header("Location", "/").finish()
}