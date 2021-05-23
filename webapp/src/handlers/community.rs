// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs


use std::env;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, ResponseError};
use actix_identity::{Identity};
use inflector::Inflector;
use serde::{Deserialize};

use qrcode_generator::QrCodeEcc;

use crate::{AppData, extract_identity_data, generate_basic_context};
use crate::models::{Communities, NewCommunity, User};
use error_handler::error_handler::CustomError;

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

#[get("/{lang}/community_index")]
pub async fn community_index(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    
    id: Identity,
    req:HttpRequest) -> impl Responder {
    
    let (mut ctx, _session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    if role != "admin".to_string() {
        let err = CustomError::new(
            406,
            "Not authorized".to_string(),
        );
        println!("{}", &err);
        return err.error_response()
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

#[get("/{lang}/open_community_index")]
pub async fn open_community_index(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    
    id: Identity,
    req:HttpRequest) -> impl Responder {
    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

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


#[get("/{lang}/community/{community_slug}")]
pub async fn view_community(
    web::Path((lang, community_slug)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    let community_select = Communities::find_from_slug(&community_slug);

    match community_select {
        Ok(community) => {

            let mut owner = false;
        
            if &session_user != "" {
                let user_select = User::find_from_slug(&session_user);

                match user_select {
                    Ok(user) => {
                        ctx.insert("user", &user);
                        
                        if community.user_id == user.id {
                            owner = true;
                        }
                    
                        // Redirect if community is closed and user isn't community owner
                        if !community.open && !owner {
                            return HttpResponse::Found().header("Location", format!("/{}", &lang)).finish()
                        };
                    },
                    Err(err) => {
                        println!("Error - {}", &err);
                        return err.error_response()
                    },
                };
        
            } else {
                if !community.open {
                    let err = CustomError::new(
                        406,
                        "Not authorized".to_string(),
                    );
                    println!("{}", &err);
                    return err.error_response()
                }
            };
        
            ctx.insert("community", &community);
            ctx.insert("owner", &owner);
            
            
            let community_add_profile_url = format!("/{}/survey_intro/{}", &lang, &community.code);
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

#[get("/{lang}/add_community")]
pub async fn add_community(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    if session_user != "" {
        let rendered = data.tmpl.render("communities/add_community.html", &ctx).unwrap();
        return HttpResponse::Ok().body(rendered)
    } else {
        let err = CustomError::new(
            406,
            "Not authorized".to_string(),
        );
        println!("{}", &err);
        return err.error_response()
    };
}

#[post("/{lang}/add_community")]
pub async fn add_community_form_input(
    web::Path(lang): web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/add_community", &lang)).finish()
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
                    return HttpResponse::Found().header("Location", format!("/{}/community/{}", &lang, community.slug)).finish()
                },
                Err(e) => {
                    println!("{}", e);
                    return e.error_response()
                }
            };
        },
        Err(err) => {
            println!("{}", &err);
            return err.error_response()
        },
    };
}

#[get("/{lang}/edit_community/{community_slug}")]
pub async fn edit_community(
    web::Path((lang, community_slug)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let (mut ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

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
                        return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, u.slug)).finish()
                    };
                },
                Err(e) => {
                    println!("Community not found. Redirecting: {}", &e);
                    return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, u.slug)).finish()
                },
            };
        },

        Err(err) => {
            println!("Error {}", &err);
            return err.error_response()
        },
    };

    let rendered = data.tmpl.render("communities/edit_community.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/edit_community/{community_slug}")]
pub async fn edit_community_form_input(
    web::Path((lang, community_slug)): web::Path<(String, String)>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/add_community", &lang)).finish()
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
                                return HttpResponse::Found().header("Location", format!("/{}/community/{}", &lang, c.slug)).finish()
                            },
                            Err(e) => {
                                println!("Community update failed: {}", e);
                                return HttpResponse::Found().header("Location", format!("/{}/edit_community/{}", &lang, &community_slug)).finish()
                            }
                        };

                    } else {
                        // redirect
                        return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
                    }
                },
                Err(e) => {
                    println!("Community not found. Redirecting: {}", &e);
                    return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, u.slug)).finish()
                }
            }
        },
        Err(err) => {
            println!("Error - {}", &err);
            return err.error_response()
        },
    };
}

#[get("/{lang}/delete_community/{code}")]
pub async fn delete_community(
    web::Path((lang, code)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (mut ctx, session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    let user = match User::find_from_slug(&session_user){
        Ok(u) => u,
        Err(e) => {
            println!("{}", e);
            return e.error_response()
        },
    };

    let community = Communities::find_from_code(&code);

    match community {
        Ok(community) => {
            if role != "admin".to_string() && community.user_id != user.id {
                // user isn't admin or the community owner
                println!("User not community owner - access denied");
                HttpResponse::Found().header("Location", format!("/{}/community_index", &lang)).finish()
            } else {

                ctx.insert("community", &community);

                let rendered = data.tmpl.render("communities/delete_community.html", &ctx).unwrap();
                return HttpResponse::Ok().body(rendered)
            }
        },
        Err(e) => {
            // no community found
            println!("{}", e);
            return e.error_response()
        }
    }
}

#[post("/{lang}/delete_community/{code}")]
pub async fn delete_community_form(
    web::Path((lang, code)): web::Path<(String, String)>,
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
            return e.error_response()
        },
    };

    let community = Communities::find_from_code(&code);

    match community {
        Ok(community) => {
            if role != "admin".to_string() && community.user_id != user.id && community.slug != "global".to_string() {
                // user isn't admin or the community owner and we're not trying to delete the global community
                let err = CustomError::new(
                    406,
                    "Not authorized".to_string(),
                );
                println!("{}", &err);
                return err.error_response()
            } else {
        
                if form.user_verify.trim().to_string() == community.tag {
                    println!("Community matches verify string - transferring users and deleting");

                    let transfer = Communities::transfer_people(community.id, &"global".to_string());

                    match transfer {
                        Ok(c) => {
                            println!("Community {} updated", c.tag);

                            // delete community
                            Communities::delete(community.id).expect("Unable to delete community");
                            return HttpResponse::Found().header("Location", format!("/{}/community_index", &lang)).finish()
                        },
                        Err(e) => {
                            println!("Community update failed: {}", e);
                            return e.error_response()
                        }
                    };
                    
                } else {
                    println!("Community does not match verify string - return to delete page");
                    return HttpResponse::Found().header("Location", format!("/{}/delete_community/{}", &lang, community.code)).finish()
                };
            }
        },
        Err(e) => {
            // no community found
            println!("{}", e);
            return e.error_response()
        }
    }
}