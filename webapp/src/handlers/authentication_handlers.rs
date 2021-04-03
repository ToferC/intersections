// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Mutex;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_session::{Session, UserSession};
use actix_identity::{Identity};
use tera::Context;
use serde::{Deserialize};

use crate::{AppData, extract_identity_data};
use crate::models::{User, verify, UserData};

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterForm {
    user_name: String,
    email: String,
    password: String,
}

#[get("/log_in")]
pub async fn login_handler(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    let rendered = data.tmpl.render("log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/log_in")]
pub async fn login_form_input(
    _data: web::Data<AppData>,
    _req: HttpRequest, 
    form: web::Form<LoginForm>,
    _session: Session,
    id: Identity,
) -> impl Responder {

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        println!("Form is empty");
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };
    
    let user = User::find_from_email(&form.email.to_lowercase().trim().to_string());

    match user {
        Ok(u) => {
            let user = u;
            println!("{:?}", &form);
        
            if verify(&user, &form.password.trim().to_string()) {
                println!("Verified");

                id.remember(user.slug.to_owned());
                        
                return HttpResponse::Found().header("Location", format!("/user/{}", user.slug)).finish()
            } else {
                // Invalid login
                println!("User not verified");
                return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
            }
        },
        _ => {
            println!("User not verified");
            return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
        },
    };

}

#[get("/register")]
pub async fn register_handler(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    let rendered = data.tmpl.render("register.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/register")]
pub async fn register_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<RegisterForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() || form.user_name.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/register")).finish()
    };

    // create user

    let user_data = UserData {
        email: form.email.to_lowercase().trim().to_owned(),
        user_name: form.user_name.trim().to_owned(),
        password: form.password.trim().to_owned(),
        role: "user".to_owned(),
        validated: false,
    };

    let user = User::create(user_data).expect("Unable to load user.");
    println!("User {} created", &user.user_name);

    let session = req.get_session();

    session.set("role", user.role.to_owned()).expect("Unable to set role cookie");
    session.set("session_user", user.slug.to_owned()).expect("Unable to set user name");

    id.remember(user.slug.to_owned());
    
    HttpResponse::Found().header("Location", format!("/user/{}", user.slug)).finish()
}

#[get("/log_out")]
pub async fn logout(
    _data: web::Data<AppData>,
    req: HttpRequest,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let session = req.get_session();

    session.clear();
    id.forget();

    HttpResponse::Found().header("Location", "/").finish()
}

