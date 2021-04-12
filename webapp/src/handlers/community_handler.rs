// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Mutex;
use std::env;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, ResponseError};
use actix_identity::{Identity};
use inflector::Inflector;
use serde::{Deserialize};

use qrcode_generator::QrCodeEcc;

use crate::{AppData, extract_identity_data, generate_basic_context};
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
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
    _req:HttpRequest) -> impl Responder {
    
    let (mut ctx, _session_user, role) = generate_basic_context(id, node_names);
    
    if role != "admin".to_string() {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/log_in").finish()
    } else {
        let communities_data = Communities::find_all();

        let communities = match communities_data {
            Ok(u) => u,
            Err(e) => {
                println!("{:?}", e);
                Vec::new()
            }
        };

        ctx.insert("communities", &communities);

        let rendered = data.tmpl.render("communities/community_index.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/open_community_index")]
pub async fn open_community_index(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
    _req:HttpRequest) -> impl Responder {
    
    let (mut ctx, _session_user, _role) = generate_basic_context(id, node_names);

    let communities_data = Communities::find_all_open();

    let communities = match communities_data {
        Ok(u) => u,
        Err(e) => {
            println!("{:?}", e);
            Vec::new()
        }
    };

    ctx.insert("communities", &communities);

    let rendered = data.tmpl.render("communities/community_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}


#[get("/community/{community_slug}")]
pub async fn view_community(
    web::Path(community_slug): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, session_user, _role) = generate_basic_context(id, node_names);
    
    let community_select = Communities::find_from_slug(&community_slug);

    match community_select {
        Ok(community) => {

            let mut owner = false;
        
            if &session_user != "" {
                let user = User::find_from_slug(&session_user).expect("Unable to load user");
        
                ctx.insert("user", &user);
                
                if community.user_id == user.id {
                    owner = true;
                }
            
                // Redirect if community is closed and user isn't community owner
                if !community.open && !owner {
                    return HttpResponse::Found().header("Location", String::from("/")).finish()
                };
            } else {
                if !community.open {
                    return HttpResponse::Found().header("Location", String::from("/")).finish()
                }
            };
        
            ctx.insert("community", &community);
            ctx.insert("owner", &owner);
            
            
            let community_add_profile_url = format!("/survey_intro/{}", &community.code);
            ctx.insert("add_community_profile_url", &community_add_profile_url);
        
            // add qr code to add profile
            let application_url: String;
            let environment = env::var("ENVIRONMENT").unwrap();
        
            if environment == "production" {
                application_url = "https://www.intersectional-data.ca".to_string();
            } else {
                application_url = "http://localhost:8088".to_string();
            };
        
            let qr = qrcode_generator::to_svg_to_string(format!("{}{}", application_url, community_add_profile_url), QrCodeEcc::Low, 245, Some("Invitation link for intersections")).unwrap();
            ctx.insert("qrcode", &qr);
        
            let rendered = data.tmpl.render("communities/view_community.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("Error: {}", err);
            return err.error_response();
        },
    };

}

#[get("/add_community")]
pub async fn add_community(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _session_user, _role) = generate_basic_context(id, node_names);

    let rendered = data.tmpl.render("communities/add_community.html", &ctx).unwrap();
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
                false,
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
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let (mut ctx, session_user, _role) = generate_basic_context(id, node_names);

    // validate user
    let user = User::find_from_slug(&session_user);
    
    match user {
        Ok(u) => {
            
            // get community
            let result = Communities::find_from_slug(&community_slug);
            
            match result {
                Ok(community) => {
                    // validate user owns community
                    if community.user_id == u.id {
                        
                        ctx.insert("community", &community);
                    
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

    let rendered = data.tmpl.render("communities/edit_community.html", &ctx).unwrap();
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
                        community.data_use_case= form.data_use_case.trim().to_owned();
                        community.open = open;

                        let update = Communities::update(&community);

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

#[get("/delete_community/{code}")]
pub async fn delete_community(
    web::Path(code): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (mut ctx, session_user, role) = generate_basic_context(id, node_names);
    
    let user = match User::find_from_slug(&session_user){
        Ok(u) => u,
        Err(e) => {
            println!("{}", e);
            User::dummy()
        },
    };

    let community = Communities::find_from_code(&code);

    match community {
        Ok(community) => {
            if role != "admin".to_string() && community.user_id != user.id {
                // user isn't admin or the community owner
                println!("User not community owner - access denied");
                HttpResponse::Found().header("Location", "/community_index").finish()
            } else {

                ctx.insert("community", &community);

                let rendered = data.tmpl.render("delete_community.html", &ctx).unwrap();
                return HttpResponse::Ok().body(rendered)
            }
        },
        Err(e) => {
            // no community found
            println!("{}", e);
            return HttpResponse::Found().header("Location", "/").finish();
        }
    }
}

#[post("/delete_community/{code}")]
pub async fn delete_community_form(
    web::Path(code): web::Path<String>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    id: Identity,
    form: web::Form<DeleteCommunityForm>,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);
    
    let user = match User::find_from_slug(&session_user){
        Ok(u) => u,
        Err(e) => {
            println!("{}", e);
            User::dummy()
        },
    };

    let community = Communities::find_from_code(&code);

    match community {
        Ok(community) => {
            if role != "admin".to_string() && community.user_id != user.id && community.slug != "global".to_string() {
                // user isn't admin or the community owner and we're not trying to delete the global community
                println!("User not community owner - access denied");
                HttpResponse::Found().header("Location", "/community_index").finish()
            } else {
        
                if form.user_verify.trim().to_string() == community.tag {
                    println!("Community matches verify string - transferring users and deleting");

                    let transfer = Communities::transfer_people(community.id, &"global".to_string());

                    match transfer {
                        Ok(c) => {
                            println!("Community {} updated", c.tag);

                            // delete community
                            Communities::delete(community.id).expect("Unable to delete community");
                            return HttpResponse::Found().header("Location", "/community_index").finish()
                        },
                        Err(e) => {
                            println!("Community update failed: {}", e);
                            return HttpResponse::Found().header("Location", 
                            format!("/delete_community/{}", community.code)).finish()
                        }
                    };
                    
                } else {
                    println!("Community does not match verify string - return to delete page");
                    return HttpResponse::Found().header("Location", format!("/delete_community/{}", community.code)).finish()
                };
            }
        },
        Err(e) => {
            // no community found
            println!("{}", e);
            return HttpResponse::Found().header("Location", "/").finish();
        }
    }
}