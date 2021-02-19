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

#[derive(Deserialize, Debug)]
pub struct DeleteUserForm {
    user_verify: String,
}

#[get("/community_index")]
pub async fn user_index(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
    id: Identity,
    _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    // validate if user is admin else redirect
    let (session_user, role) = extract_identity_data(&id);
    
    if role != "admin".to_string() {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/log_in").finish()
    } else {

        ctx.insert("session_user", &session_user);
        ctx.insert("role", &role);

        // add node_names for navbar drop down
        ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

        let user_data = User::find_all();

        let users = match user_data {
            Ok(u) => u,
            Err(e) => {
                println!("{:?}", e);
                Vec::new()
            }
        };

        ctx.insert("users", &users);

        let rendered = data.tmpl.render("user_index.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}


#[get("/view_community/{community_slug}")]
pub async fn user_page_handler(
    web::Path(community_slug): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let mut ctx = Context::new();

    // validate if user == user_name  or admin else redirect
    let (session_user, role) = extract_identity_data(&id);
    
    if session_user.to_lowercase() != slug.to_lowercase() && role != "admin".to_string() {
        HttpResponse::Found().header("Location", "/log_in").finish()
    } else {

        ctx.insert("session_user", &session_user);
        ctx.insert("role", &role);

        // add node_names for navbar drop down
        ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
        let user = User::find_from_slug(&slug).expect("Could not load user");
    
        ctx.insert("user", &user);
    
        let rendered = data.tmpl.render("user_page.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/add_community")]
pub async fn add_community_handler(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
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

#[post("/add_community")]
pub async fn add_community_form_input(
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
    };

    let user = User::create(user_data).expect("Unable to load user.");
    println!("User {} created", &user.user_name);

    let session = req.get_session();

    session.set("role", user.role.to_owned()).expect("Unable to set role cookie");
    session.set("session_user", user.slug.to_owned()).expect("Unable to set user name");

    id.remember(user.slug.to_owned());
    
    HttpResponse::Found().header("Location", format!("/user/{}", user.slug)).finish()
}

#[get("/delete_community/{target_id}")]
pub async fn delete_community_handler(
    web::Path(target_id): web::Path<i32>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
    _req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);
    
    if role != "admin".to_string() {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/").finish()
    } else {

        let user = User::find(target_id);
        
        match user {
            Ok(u) => {
                let mut ctx = Context::new();

                ctx.insert("user", &u);
            
                ctx.insert("session_user", &session_user);
                ctx.insert("role", &role);

                // add node_names for navbar drop down
                ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

                let rendered = data.tmpl.render("delete_user.html", &ctx).unwrap();
                return HttpResponse::Ok().body(rendered)
            },
            Err(c) => {
                // no user returned for ID
                println!("{}", c);
                return HttpResponse::Found().header("Location", "/user_index").finish()
            },
        }
    }
}

#[post("/delete_community/{target_id}")]
pub async fn delete_community(
    web::Path(target_id): web::Path<i32>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    id: Identity,
    form: web::Form<DeleteUserForm>,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);
    
    if role != "admin".to_string() {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/").finish()
    } else {

        let user = User::find(target_id);
        
        match user {
            Ok(u) => {
                if form.user_verify.trim().to_string() == u.user_name {
                    println!("User matches verify string - deleting");
                    // forget id if delete target is user
                    if session_user == u.slug {
                        id.forget();
                    };
                    // delete user
                    User::delete(u.id).expect("Unable to delete user");
                    return HttpResponse::Found().header("Location", "/user_index").finish()
                } else {
                    println!("User does not match verify string - return to delete page");
                    return HttpResponse::Found().header("Location", format!("/delete_user/{}", u.id)).finish()
                };
            },
            Err(c) => {
                // no user returned for ID
                println!("{}", c);
                return HttpResponse::Found().header("Location", "/user_index").finish()
            },
        }
    }
}