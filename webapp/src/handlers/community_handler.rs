// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::path::Path;

use std::sync::Mutex;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use tera::Context;
use serde::{Deserialize};

use qrcode_generator::QrCodeEcc;

use crate::{AppData, extract_identity_data};
use crate::models::{Communities, NewCommunity, User};

#[derive(Deserialize, Debug)]
pub struct CommunityForm {
    community_name: String,
    description: String,
    open: bool,
}

#[derive(Deserialize, Debug)]
pub struct DeleteCommunityForm {
    user_verify: String,
}

#[get("/community_index")]
pub async fn community_index(
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

        let communities_data = Communities::find_all();

        let communities = match communities_data {
            Ok(u) => u,
            Err(e) => {
                println!("{:?}", e);
                Vec::new()
            }
        };

        ctx.insert("communities", &communities);

        let rendered = data.tmpl.render("community_index.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}


#[get("/community/{community_slug}")]
pub async fn view_community(
    web::Path(community_slug): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let mut ctx = Context::new();

    // validate if user == user_name  or admin else redirect
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let host_name = req.app_config().host();

    // qr_code
    if !Path::new(&format!("webapp/static/tmp/{}.png",community_slug)).exists() {
        qrcode_generator::to_png_to_file(format!("https://{}/community/{}", host_name, &community_slug), QrCodeEcc::Low, 1024, format!("webapp/static/tmp/{}.png",community_slug)).unwrap();
    };

    ctx.insert("qrcode", &format!("/static/tmp/{}.png",community_slug));

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    let community = Communities::find_from_slug(&community_slug).expect("Could not load community");
    ctx.insert("community", &community);

    let rendered = data.tmpl.render("view_community.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/add_community")]
pub async fn add_community(
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
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/add_community")).finish()
    };

    // create community

    let community_data = NewCommunity::new(
        form.community_name.trim().to_owned(), 
        form.description.trim().to_owned(), 
        form.open.to_owned()
    );

    let community = Communities::create(&community_data);

    match community {
        Ok(community) => {

            println!("Community {} created", community.tag);

            // add community to user
            let user = User::find_from_slug(&id.identity().unwrap());
        
            match user {
                Ok(u) => {
                    let mut user = u;
        
                    user.managed_communities.push(community.id);
        
                    let result = User::update(user);
        
                    match result {
                        Ok(_u) => println!("User updated"),
                        Err(e) => println!("{}", e),
                    };
                                
                        return HttpResponse::Found().header("Location", format!("/community/{}", community.slug)).finish()
                },
                _ => {
                    println!("User not verified");
                    return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
                },
            };
        },
        Err(e) => {
            println!("{}", e);
            return HttpResponse::Found().header("Location", String::from("/add_community")).finish()
        }
    };
}

#[get("/edit_community")]
pub async fn edit_community(
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

#[post("/edit_community")]
pub async fn edit_community_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/add_community")).finish()
    };

    // create community

    let community_data = NewCommunity::new(
        form.community_name.trim().to_owned(), 
        form.description.trim().to_owned(), 
        form.open.to_owned()
    );

    let community = Communities::create(&community_data);

    match community {
        Ok(community) => {

            println!("Community {} created", community.tag);

            // add community to user
            let user = User::find_from_slug(&id.identity().unwrap());
        
            match user {
                Ok(u) => {
                    let mut user = u;
        
                    user.managed_communities.push(community.id);
        
                    let result = User::update(user);
        
                    match result {
                        Ok(_u) => println!("User updated"),
                        Err(e) => println!("{}", e),
                    };
                                
                        return HttpResponse::Found().header("Location", format!("/community/{}", community.slug)).finish()
                },
                _ => {
                    println!("User not verified");
                    return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
                },
            };
        },
        Err(e) => {
            println!("{}", e);
            return HttpResponse::Found().header("Location", String::from("/add_community")).finish()
        }
    };
}

#[get("/delete_community/{target_id}")]
pub async fn delete_community(
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
pub async fn delete_community_form(
    web::Path(target_id): web::Path<i32>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    id: Identity,
    form: web::Form<DeleteCommunityForm>,
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