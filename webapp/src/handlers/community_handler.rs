// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::{path::Path, println};

use std::sync::Mutex;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_identity::{Identity};
use inflector::Inflector;
use tera::Context;
use serde::{Deserialize};

use qrcode_generator::QrCodeEcc;

use crate::{AppData, extract_identity_data};
use crate::models::{Communities, NewCommunity, User};

#[derive(Deserialize, Debug)]
pub struct CommunityForm {
    community_name: String,
    description: String,
    data_use_case: String,
    open: String,
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

    let community_url = format!("/community/{}", &community_slug);

    /*
    if !Path::new(&format!("webapp/static/tmp/{}.png",community_slug)).exists() {
        qrcode_generator::to_png_to_file(&community_url, QrCodeEcc::Low, 1024, format!("webapp/static/tmp/{}.png",community_slug)).unwrap();
    };

    ctx.insert("qrcode", &format!("/static/tmp/{}.png",community_slug));
    */

    
    // qr_code
    ctx.insert("community_url", &community_url);
    
    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
    let community = Communities::find_from_slug(&community_slug).expect("Could not load community");
    ctx.insert("community", &community);
    
    let community_add_profile_url = format!("/survey_intro/{}", &community.code);
    ctx.insert("add_community_profile_url", &community_add_profile_url);

    // add qr code to add profile

    let host = req.app_config().host();
    println!("{}", host);

    let qr = qrcode_generator::to_svg_to_string(format!("https://{}{}", host, community_add_profile_url), QrCodeEcc::Low, 245, Some("Invitation link for intersections")).unwrap();
    ctx.insert("qrcode", &qr);

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

    let rendered = data.tmpl.render("add_community.html", &ctx).unwrap();
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

    // validate user
    let user = User::find_from_slug(&id.identity().unwrap());
        
    match user {
        Ok(u) => {

            let open = match form.open.as_str() {
                "open" => true,
                _ => false,
            };
            
            // create community
            let community_data = NewCommunity::new(
                form.community_name.trim().to_owned(), 
                form.description.trim().to_owned(),
                form.data_use_case.trim().to_owned(),
                u.email.to_owned(),
                open,
                u.id,
            );
            
            let community = Communities::create(&community_data);
        
            match community {
                Ok(community) => {
                    println!("Community {} created", &community.tag);
                    return HttpResponse::Found().header("Location", format!("/community/{}", community.slug)).finish()
                },
                Err(e) => {
                    println!("{}", e);
                    return HttpResponse::Found().header("Location", String::from("/add_community")).finish()
                }
            };
        },
        _ => {
            println!("User not verified");
            return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
        },
    };
}

#[get("/edit_community/{community_slug}")]
pub async fn edit_community(
    web::Path(community_slug): web::Path<String>,
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

    // validate user
    let user = User::find_from_slug(&id.identity().unwrap());
    
    match user {
        Ok(u) => {
            
            // get community
            let result = Communities::find_from_slug(&community_slug);
            
            match result {
                Ok(mut community) => {
                    // validate user owns community
                    if community.user_id == u.id {
                        
                        ctx.insert("community", &community);
                    
                        // add node_names for navbar drop down
                        ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
                    
                    } else {
                        // user does not own community - redirect
                        return HttpResponse::Found().header("Location", format!("/user/{}", u.slug)).finish()
                    };
                },
                Err(e) => {
                    println!("Community not found. Redirecting: {}", &e);
                    return HttpResponse::Found().header("Location", format!("/user/{}", u.slug)).finish()
                },
            };
        },

        _ => {
            // user not found
            println!("User not verified");
            return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
        },
    };

    let rendered = data.tmpl.render("edit_community.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/edit_community/{community_slug}")]
pub async fn edit_community_form_input(
    web::Path(community_slug): web::Path<String>,
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

    // validate user
    let user = User::find_from_slug(&id.identity().unwrap());
        
    match user {
        Ok(u) => {

            // get community
            let result = Communities::find_from_slug(&community_slug);

            match result {
                Ok(mut community) => {
                    // validate user owns community
                    if community.user_id == u.id {

                        let open = match form.open.as_str() {
                            "open" => true,
                            _ => false,
                        };

                        // update community
                        community.tag = form.community_name.trim().to_owned();
                        community.slug = form.community_name.trim().to_snake_case().to_owned();
                        community.description = form.description.trim().to_owned();
                        community.open = open;

                        let update = Communities::update(community);

                        match update {
                            Ok(c) => {
                                println!("Community {} updated", c.tag);
                                return HttpResponse::Found().header("Location", format!("/community/{}", c.slug)).finish()
                            },
                            Err(e) => {
                                println!("Community update failed: {}", e);
                                return HttpResponse::Found().header("Location", String::from("/edit_community")).finish()
                            }
                        };

                    } else {
                        // redirect
                        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
                    }
                },
                Err(e) => {
                    println!("Community not found. Redirecting: {}", &e);
                    return HttpResponse::Found().header("Location", format!("/user/{}", u.slug)).finish()
                }
            }
        },
        _ => {
            println!("User not verified");
            return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
        },
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