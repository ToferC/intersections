// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs


use std::env;
use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, ResponseError, Responder, get, post, web};
use actix_identity::Identity;
use error_handler::error_handler::CustomError;
use inflector::Inflector;
use serde::Deserialize;

use qrcode_generator::QrCodeEcc;

use crate::{AppData, extract_identity_data, generate_basic_context, models::Phrases};
use crate::models::{Communities, NewCommunity, User, generate_phrases, translate_phrases};

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
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    id: Option<Identity>,
    req:HttpRequest) -> impl Responder {

    let lang = path.into_inner();
    
    let (mut ctx, _session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    if role != "admin".to_string() {
        let err = error_handler::error_handler::CustomError::new(
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

        let mut phrase_ids = Vec::new();

        for c in &communities {
            phrase_ids.push(c.tag);
            phrase_ids.push(c.description);
            phrase_ids.push(c.data_use_case);
        };

        let phrase_map = Phrases::get_phrase_map(phrase_ids, &lang).expect("Unable to get phrase mamp");

        ctx.insert("phrases", &phrase_map);

        ctx.insert("communities", &communities);

        let rendered = data.tmpl.render("communities/community_index.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/{lang}/open_community_index")]
pub async fn open_community_index(
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    id: Option<Identity>,
    req:HttpRequest) -> impl Responder {

    let lang = path.into_inner();
    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    let communities_data = Communities::find_all_open();

    
    let communities = match communities_data {
        Ok(u) => u,
        Err(e) => {
            println!("{:?}", e);
            Vec::new()
        }
    };
    
    let mut phrase_ids = Vec::new();

    for c in &communities {
        phrase_ids.push(c.tag);
        phrase_ids.push(c.description);
        phrase_ids.push(c.data_use_case);
    };

    let phrase_map = Phrases::get_phrase_map(phrase_ids, &lang).expect("Unable to get phrase mamp");

    ctx.insert("phrases", &phrase_map);

    ctx.insert("communities", &communities);

    let rendered = data.tmpl.render("communities/community_index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}


#[get("/{lang}/community/{community_slug}")]
pub async fn view_community(
    path: web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (lang, community_slug) = path.into_inner();

    let (mut ctx, session_user, _role, _lang) = generate_basic_context(Some(id), &lang, req.uri().path());
    
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
        
            let phrase_map = community.get_phrases(&lang);

            ctx.insert("community", &community);
            ctx.insert("phrases", &phrase_map);
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
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let lang = path.into_inner();
    
    let (ctx, session_user, _role, _lang) = generate_basic_context(Some(id), &lang, req.uri().path());

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
    path: web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {

    let lang = path.into_inner();

    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().append_header(("Location", format!("/{}/add_community", &lang))).finish()
    };

    // validate user
    let user = User::find_from_slug(&id.id().unwrap());
        
    match user {
        Ok(u) => {

            let open = match form.open.as_str() {
                "open" => true,
                _ => false,
            };

            let mut texts = Vec::new();

            texts.push(form.community_name.trim().to_owned());
            texts.push(form.description.to_lowercase().trim().to_owned());
            texts.push(form.data_use_case.to_lowercase().trim().to_owned());

            let phrases = generate_phrases(texts, &lang)
                .await
                .expect("Unable to generate phrases");

            let tm = Arc::new(phrases.clone());
            let l = Arc::new(lang.clone());

            let _translate = tokio::spawn(translate_phrases(tm, l));
            
            // create community
            let community_data = NewCommunity::new(
                phrases[0].0, 
                phrases[1].0,
                phrases[2].0,
                u.email.to_owned(),
                open,
                phrases[0].1.to_snake_case(),
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
    path: web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let (lang, community_slug) = path.into_inner();

    let (mut ctx, session_user, _role, _lang) = generate_basic_context(Some(id), &lang, req.uri().path());

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
                        let phrase_map = community.get_phrases(&lang);
                        ctx.insert("phrases", &phrase_map);
                    
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
    path: web::Path<(String, String)>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<CommunityForm>,
    id: Identity,
) -> impl Responder {
    let (lang, community_slug) = path.into_inner();

    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.community_name.is_empty() || form.description.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/add_community", &lang)).finish()
    };

    // validate user
    let user = User::find_from_slug(&id.id().unwrap());
        
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

                        let mut texts = Vec::new();

                        texts.push(form.community_name.trim().to_owned());
                        texts.push(form.description.to_lowercase().trim().to_owned());
                        texts.push(form.data_use_case.to_lowercase().trim().to_owned());

                        let phrases = generate_phrases(texts, &lang)
                            .await
                            .expect("Unable to generate phrases");

                        let tm = Arc::new(phrases.clone());
                        let l = Arc::new(lang.clone());

                        let _translate = tokio::spawn(translate_phrases(tm, l));

                        // update community
                        community.tag = phrases[0].0;
                        community.description = phrases[1].0;
                        community.data_use_case= phrases[2].0;
                        community.open = open;
                        community.slug = phrases[0].1.trim().to_snake_case().to_owned();

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
    path: web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req: HttpRequest,
    id: Identity,
) -> impl Responder {
    let (lang, code) = path.into_inner();

    let (mut ctx, session_user, role, _lang) = generate_basic_context(Some(id), &lang, req.uri().path());
    
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
                let phrase_map = community.get_phrases(&lang);
                ctx.insert("phrases", &phrase_map);

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
    path: web::Path<(String, String)>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    id: Identity,
    form: web::Form<DeleteCommunityForm>,
) -> impl Responder {
    let (lang, code) = path.into_inner();

    let (session_user, role, id) = extract_identity_data(Some(id));
    
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
        
                if form.user_verify.trim().to_string() == Phrases::find(community.tag, &lang).expect("Unable to find phrase").text {
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