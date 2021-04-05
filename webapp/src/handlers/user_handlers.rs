// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Mutex;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use inflector::Inflector;
use tera::Context;
use serde::{Deserialize};

use crate::{AppData, extract_identity_data};
use crate::models::{User, Communities};

#[derive(Deserialize, Debug)]
pub struct UserForm {
    user_name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteUserForm {
    user_verify: String,
}

#[get("/user_index")]
pub async fn user_index(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
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

        let rendered = data.tmpl.render("users/user_index.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/user/{slug}")]
pub async fn user_page_handler(
    web::Path(slug): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
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

        let c = Communities::find_by_owner_user_id(&user.id);

        let communities = match c {
            Ok(c) => c,
            Err(e) => {
                println!("Error - {}", e);
                Vec::new()
            }
        };

        ctx.insert("communities", &communities);
    
        let rendered = data.tmpl.render("users/user_page.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/edit_user/{slug}")]
pub async fn edit_user(
    data: web::Data<AppData>,
    web::Path(slug): web::Path<String>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let user = User::find_from_slug(&slug);

    match user {
        Ok(user) => {

            if &user.slug != &session_user && role != "admin" {
                // action not allowed
                return HttpResponse::Found().header("Location", "/log_in").finish()
            };

            ctx.insert("user", &user);

            // add node_names for navbar drop down
            ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
        
            let rendered = data.tmpl.render("users/edit_user.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("Error: No user found - {}", err);
            return HttpResponse::Found().header("Location", "/log_in").finish()
        },
    };
}

#[post("/edit_user/{slug}")]
pub async fn edit_user_post(
    _data: web::Data<AppData>,
    web::Path(slug): web::Path<String>,
    _req: HttpRequest, 
    form: web::Form<UserForm>,
    id: Identity,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);

    if form.email.is_empty() || 
    form.user_name.is_empty() ||
    &session_user != &slug ||
    &role != "admin" {
        // validate form has data or and permissions exist
        return HttpResponse::Found().header("Location", format!("/edit_user/{}", &slug)).finish()
    };

    // update user
    let user = User::find_from_slug(&slug);

    match user {
        Ok(mut user) => {

            let mut email_changed = false;
            let mut user_name_changed = false;

            // update user
            if &form.email.to_lower_case().trim() != &user.email {
                user.email = form.email.to_lowercase().trim().to_owned();
                user.validated = false;
                email_changed = true;
            };

            if &form.user_name.trim() != &user.user_name {
                user.user_name = form.user_name.trim().to_owned();
                user.slug = user.user_name.clone().to_snake_case();
                
                id.forget();
                id.remember(user.slug.to_owned());
                user_name_changed = true;
            };

            if email_changed || user_name_changed {
                // changes registered, update user
                let user = User::update(user).expect("Unable to update user.");
                println!("User {} updated", &user.user_name);

                if email_changed {
                    // validate email
                    return HttpResponse::Found().header("Location", "/email_verification").finish()
                } else {
                    // return to user page
                    return HttpResponse::Found().header("Location", format!("/user/{}", &user.slug)).finish()
                }
            } else {
                // no change
                return HttpResponse::Found().header("Location", format!("/user/{}", &user.slug)).finish()
            };
        },
        Err(err) => {
            println!("Error - user not found{}", err);
            return HttpResponse::Found().header("Location", "/user_index").finish()
        },
    };
}

#[get("/delete_user/{slug}")]
pub async fn delete_user_handler(
    web::Path(slug): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);
    
    if role != "admin".to_string() && &session_user != &slug {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/").finish()
    } else {

        let user = User::find_from_slug(&slug);
        
        match user {
            Ok(u) => {
                let mut ctx = Context::new();

                ctx.insert("user", &u);
            
                ctx.insert("session_user", &session_user);
                ctx.insert("role", &role);

                // add node_names for navbar drop down
                ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

                let rendered = data.tmpl.render("users/delete_user.html", &ctx).unwrap();
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

#[post("/delete_user/{target_id}")]
pub async fn delete_user(
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

                    // transfer people from user communities to global community
                    let communities = Communities::find_by_owner_user_id(&u.id).expect("Unable to load communities");

                    for community in communities {
                        Communities::transfer_people(community.id, &"global".to_string()).expect("Unable to transfer people");
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