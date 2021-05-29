// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, ResponseError};
use actix_identity::{Identity};
use inflector::Inflector;
use serde::{Deserialize};

use crate::{AppData, extract_identity_data, generate_basic_context};
use crate::models::{User, Communities, Phrases};
use crate::handlers::DeleteForm;
use error_handler::error_handler::CustomError;

#[derive(Deserialize, Debug)]
pub struct UserForm {
    user_name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
pub struct AdminUserForm {
    user_name: String,
    email: String,
    role: String,
    validated: String,
}

#[get("/{lang}/user_index")]
pub async fn user_index(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
    
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

#[get("/{lang}/user/{slug}")]
pub async fn user_page_handler(
    web::Path((lang, slug)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    if session_user.to_lowercase() != slug.to_lowercase() && role != "admin".to_string() {
        let err = CustomError::new(
            406,
            "Not authorized".to_string(),
        );
        println!("{}", &err);
        return err.error_response()

    } else {
    
        let user_select = User::find_from_slug(&slug);

        match user_select {
            Ok(user) => {
                ctx.insert("user", &user);
        
                let c = Communities::find_by_owner_user_id(&user.id);
        
                let communities = match c {
                    Ok(c) => c,
                    Err(e) => {
                        println!("Error - {}", e);
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
            
                let rendered = data.tmpl.render("users/user_page.html", &ctx).unwrap();
                HttpResponse::Ok().body(rendered)
            },
            Err(err) => {
                println!("{}", &err);
                return err.error_response()
            },
        }
    
    }
}

#[get("/{lang}/edit_user/{slug}")]
pub async fn edit_user(
    data: web::Data<AppData>,
    web::Path((lang, slug)): web::Path<(String, String)>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    let user = User::find_from_slug(&slug);

    match user {
        Ok(user) => {

            if &user.slug != &session_user && role != "admin" {
                let err = CustomError::new(
                    406,
                    "Not authorized".to_string(),
                );
                println!("{}", &err);
                return err.error_response()
            };

            ctx.insert("user", &user);
        
            let rendered = data.tmpl.render("users/edit_user.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("{}", &err);
            return err.error_response()
        },
    };
}

#[post("/{lang}/edit_user_post/{slug}")]
pub async fn edit_user_post(
    _data: web::Data<AppData>,
    web::Path((lang, slug)): web::Path<(String, String)>,
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
        return HttpResponse::Found().header("Location", format!("/{}/edit_user/{}", &lang, &slug)).finish()
    };

    // update user
    let user = User::find_from_slug(&slug);

    match user {
        Ok(mut user) => {

            let mut email_changed = false;
            let mut user_name_changed = false;

            // update user email
            if &form.email.to_lower_case().trim() != &user.email {
                user.email = form.email.to_lowercase().trim().to_owned();
                user.validated = false;
                email_changed = true;

                // update contact email in owned communities
                let communities = Communities::find_by_owner_user_id(&user.id).expect("Unable to load communities");

                for mut c in communities {
                    c.contact_email = user.email.to_owned();
                    Communities::update(&c).expect("Unable to update community");
                };
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
                // changes registered, update user
                let user_update = User::update(user);

                match user_update {
                    Ok(user) => {
                        println!("User {} updated", &user.user_name);
        
                        if email_changed {
                            // validate email
                            return HttpResponse::Found().header("Location", "/email_verification").finish()
                        } else {
                            // return to user page
                            return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, &user.slug)).finish()
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                        return err.error_response()
                    },
                };
            } else {
                // no change
                return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, &user.slug)).finish()
            };
        },
        Err(err) => {
            println!("Error - {}", err);
            return err.error_response()
        },
    };
}

#[get("/{lang}/admin_edit_user/{slug}")]
pub async fn admin_edit_user(
    data: web::Data<AppData>,
    web::Path((lang, slug)): web::Path<(String, String)>,
    
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, _session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    if &role != &"admin".to_string() {
        let err = CustomError::new(
            406,
            "Not authorized".to_string(),
        );
        println!("{}", &err);
        return err.error_response()
    };

    let user = User::find_from_slug(&slug);

    match user {
        Ok(user) => {

            ctx.insert("user", &user);
        
            let rendered = data.tmpl.render("users/admin_edit_user.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("{}", &err);
            return err.error_response()
        },
    };
}

#[post("/{lang}/admin_edit_user/{slug}")]
pub async fn admin_edit_user_post(
    _data: web::Data<AppData>,
    web::Path((lang, slug)): web::Path<(String, String)>,
    _req: HttpRequest, 
    form: web::Form<AdminUserForm>,
    id: Identity,
) -> impl Responder {

    let (_session_user, role) = extract_identity_data(&id);

    if form.email.is_empty() || 
    form.user_name.is_empty() ||
    &role != "admin" {
        // validate form has data or and permissions exist
        return HttpResponse::Found().header("Location", format!("/{}/admin_edit_user/{}", &lang, &slug)).finish()
    };

    // update user
    let user = User::find_from_slug(&slug);

    match user {
        Ok(mut user) => {

            let mut validated: bool = false;

            if form.validated == "true" {
                validated = true;
            };

            user.validated = validated;
            user.role = form.role.to_lower_case().trim().to_owned();

            // update user email
            if &form.email.to_lower_case().trim() != &user.email {
                user.email = form.email.to_lowercase().trim().to_owned();

                // update contact email in owned communities
                let communities = Communities::find_by_owner_user_id(&user.id).expect("Unable to load communities");

                for mut c in communities {
                    c.contact_email = user.email.to_owned();
                    Communities::update(&c).expect("Unable to update community");
                };
            };

            if &form.user_name.trim() != &user.user_name {
                user.user_name = form.user_name.trim().to_owned();
                user.slug = user.user_name.clone().to_snake_case();
            };

            
            // changes registered, update user
            let user_update = User::update(user);

            match user_update {
                Ok(user) => {
                    println!("User {} updated", &user.user_name);
    
                    // return to user page
                    return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, &user.slug)).finish()
                },
                Err(err) => {
                    println!("{}", err);
                    return err.error_response()
                },
            };
        },
        Err(err) => {
            println!("Error - {}", err);
            return err.error_response()
        },
    };
}

#[get("/{lang}/delete_user/{slug}")]
pub async fn delete_user_handler(
    web::Path((lang, slug)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    
    req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (mut ctx, session_user, role, _lang) = generate_basic_context(id, &lang, req.uri().path());
    
    if role != "admin".to_string() && &session_user != &slug {
        println!("User not admin");
        HttpResponse::Found().header("Location", "/").finish()
    } else {

        let user = User::find_from_slug(&slug);
        
        match user {
            Ok(u) => {

                ctx.insert("user", &u);
            
                let rendered = data.tmpl.render("users/delete_user.html", &ctx).unwrap();
                return HttpResponse::Ok().body(rendered)
            },
            Err(err) => {
                // no user returned for ID
                println!("{}", err);
                return err.error_response()
            },
        }
    }
}

#[post("/{lang}/delete_user/{slug}")]
pub async fn delete_user(
    web::Path((lang, slug)): web::Path<(String, String)>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    id: Identity,
    form: web::Form<DeleteForm>,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);
    
    if session_user.to_lowercase() != slug.to_lowercase() && role != "admin".to_string() {
        let err = CustomError::new(
            406,
            "Not authorized".to_string(),
        );
        println!("{}", &err);
        return err.error_response()
    } else {

        let user = User::find_from_slug(&slug);
        
        match user {
            Ok(u) => {
                if form.verify.trim().to_string() == u.user_name {
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
                    return HttpResponse::Found().header("Location", format!("/{}/user_index", &lang)).finish()
                } else {
                    println!("User does not match verify string - return to delete page");
                    return HttpResponse::Found().header("Location", format!("/{}/delete_user/{}", &lang, u.id)).finish()
                };
            },
            Err(err) => {
                // no user returned for ID
                println!("{}", err);
                return err.error_response()
            },
        }
    }
}