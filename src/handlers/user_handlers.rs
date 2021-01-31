use std::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use tera::Context;
use serde::{Deserialize, Serialize};

use crate::AppData;
use crate::models::{User, SlimUser, LoggedUser, InsertableUser, UserData, Nodes};
use crate::error_handler::CustomError;

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterForm {
    user_name: String,
    email: String,
    community_name: String,
    password: String,
}

#[get("/log_in")]
pub async fn login_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    let rendered = data.tmpl.render("log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/log_in")]
pub async fn login_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };

    HttpResponse::Found().header("Location", "/").finish()
}

#[get("/register")]
pub async fn register_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    
    let mut ctx = Context::new();

    let node_vec = Nodes::find_all_linked_names().expect("Unable to load nodes");
    ctx.insert("node_names", &node_vec);

    let user = User

    let rendered = data.tmpl.render("register.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/register")]
pub async fn register_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<RegisterForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() || form.community_name.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/register")).finish()
    };

    HttpResponse::Found().header("Location", "/").finish()
}

#[post("/log_out")]
pub async fn logout(
    _data: web::Data<AppData>,
    req: HttpRequest, 
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    HttpResponse::Found().header("Location", "/").finish()
}