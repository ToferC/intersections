// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Mutex;
use std::env;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_session::{Session, UserSession};
use actix_identity::{Identity};
use serde::{Deserialize};

use crate::{AppData, generate_basic_context, generate_email_context, extract_identity_data};
use crate::models::{User, verify, UserData, EmailVerification, 
    InsertableVerification, Email, PasswordResetToken, 
    InsertablePasswordResetToken};

use super::EmailForm;

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
pub struct VerifyForm {
    code: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordForm {
    password: String,
}

#[get("/{lang}/log_in")]
pub async fn login_handler(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let rendered = data.tmpl.render("authentication/log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/log_in")]
pub async fn login_form_input(
    web::Path(lang): web::Path<String>,
    _data: web::Data<AppData>,
    _req: HttpRequest, 
    form: web::Form<LoginForm>,
    _session: Session,
    id: Identity,
) -> impl Responder {

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        println!("Form is empty");
        return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
    };
    
    let user = User::find_from_email(&form.email.to_lowercase().trim().to_string());

    match user {
        Ok(u) => {
            let user = u;
            println!("{:?}", &form);
        
            if verify(&user, &form.password.trim().to_string()) {
                println!("Verified");

                id.remember(user.slug.to_owned());
                        
                return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, user.slug)).finish()
            } else {
                // Invalid login
                println!("User not verified");
                return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
            }
        },
        _ => {
            println!("User not verified");
            return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
        },
    };

}

#[get("/{lang}/register")]
pub async fn register_handler(
     web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let rendered = data.tmpl.render("authentication/register.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/registration_error")]
pub async fn registration_error(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let rendered = data.tmpl.render("authentication/registration_error.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/register")]
pub async fn register_form_input(
    web::Path(lang): web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<RegisterForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() || form.user_name.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/register", &lang)).finish()
    };

    // create user

    let user_data = UserData {
        email: form.email.to_lowercase().trim().to_owned(),
        user_name: form.user_name.trim().to_owned(),
        password: form.password.trim().to_owned(),
        role: "user".to_owned(),
        validated: false,
    };

    let insert_user = User::create(user_data);

    match insert_user {
        Ok(user) => {
            println!("User {} created", &user.user_name);
        
            let session = req.get_session();
        
            session.set("role", user.role.to_owned()).expect("Unable to set role cookie");
            session.set("session_user", user.slug.to_owned()).expect("Unable to set user name");
        
            id.remember(user.slug.to_owned());
            
            return HttpResponse::Found().header("Location", format!("/{}/email_verification", &lang)).finish()
        },
        Err(err) => {
            println!("Error: {}", err);
            return HttpResponse::Found().header("Location", format!("/{}/registration_error", &lang)).finish()
        },
    };
}

#[get("/{lang}/email_verification")]
pub async fn email_verification(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (mut ctx, session_user, role, _lang) = generate_basic_context(id.clone(), &lang, req.uri().path(), node_names);

    if session_user == "".to_string() && role != "admin".to_string() {
        // person signed in shouldn't be here
        return HttpResponse::Found().header("Location", format!("/{}", &lang)).finish()
    };

    let user = User::find_from_slug(&session_user);

    match user {
        Ok(user) => {
            ctx.insert("user", &user);

            // send verification email
            let verification = EmailVerification::create(
                &InsertableVerification::new(&user.email)
            ).expect("Unable to create verification");

            let (mut email_ctx, _, _, _) = generate_email_context(id, &lang, req.uri().path());
            email_ctx.insert("user", &user);
            email_ctx.insert("verification", &verification);

            let rendered_email = data.tmpl.render("emails/email_verification.html", &email_ctx).unwrap();

            let email = Email::new(
                user.email.clone(), 
                rendered_email, 
                "Email Verification Code - Intersectional Data".to_string(), 
                data.mail_client.clone(),
            );

            let r = Email::send(&email).await;

            match r {
                Ok(()) => println!("Email sent"),
                Err(err) => println!("Error {}", err),
            };
        
            let rendered = data.tmpl.render("authentication/email_verification.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("No user found: {}", err);
            return HttpResponse::Found().header("Location", format!("/{}", &lang)).finish()
        },
    }
}

#[post("/{lang}/verify_code")]
pub async fn verify_code(
    web::Path(lang): web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<VerifyForm>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // Get session data and add to context
    let (session_user, _role) = extract_identity_data(&id);

    // validate form has data or re-load form
    if form.code.is_empty() || session_user == "".to_string() {
        return HttpResponse::Found().header("Location", format!("/{}/email_verification", &lang)).finish()
    };

    // load user
    let mut user = User::find_from_slug(&session_user).expect("Unable to load user");

    let verification_code = EmailVerification::find_by_email(&user.email).expect("Unable to load email verification");

    // verify code entered vs code in email
    if form.code.trim() != verification_code.activation_code || chrono::Utc::now().naive_local() > verification_code.expires_on {
        // code doesn't match or time expired
        return HttpResponse::Found().header("Location", format!("/{}/email_verification", &lang)).finish()
    };
    
    // validate user
    user.validated = true;
    let user = User::update(user).expect("Unable to update user");

    // delete email_verification
    EmailVerification::delete(verification_code.id).expect("Unable to delete verification code");
    
    HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, user.slug)).finish()
}

#[get("/{lang}/log_out")]
pub async fn logout(
     web::Path(lang): web::Path<String>,
    _data: web::Data<AppData>,
    req: HttpRequest,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let session = req.get_session();

    session.clear();
    id.forget();

    HttpResponse::Found().header("Location", format!("/{}", &lang)).finish()
}

#[get("/{lang}/request_password_reset")]
pub async fn request_password_reset(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let rendered = data.tmpl.render("authentication/request_password_reset.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/request_password_reset")]
pub async fn request_password_reset_post(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<EmailForm>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // Get session data and add to context
    let (_session_user, _role) = extract_identity_data(&id);

    // validate form has data or re-load form
    if form.email.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/email_verification", &lang)).finish()
    };

    // load user
    let user = User::find_from_email(&form.email.trim().to_string());

    match user {
        Ok(user) => {

            // create token
            let token = PasswordResetToken::create(
                &InsertablePasswordResetToken::new(&user.email)
            ).expect("Unable to create verification");

            // render email
            let (mut email_ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

            email_ctx.insert("user", &user);
            email_ctx.insert("verification", &token);

            // add application email link
            let application_url: String;
            let environment = env::var("ENVIRONMENT").unwrap();

            if environment == "production" {
                application_url = "https://www.intersectional-data.ca".to_string();
            } else {
                application_url = "http://localhost:8088".to_string();
            };

            email_ctx.insert("application_url", &application_url);

            email_ctx.insert("token", &token.reset_token);

            let rendered_email = data.tmpl.render("emails/password_reset_email.html", &email_ctx).unwrap();

            let email = Email::new(
                user.email.clone(), 
                rendered_email, 
                "Password Reset - Intersectional Data".to_string(), 
                data.mail_client.clone(),
            );

            // send email
            let r = Email::send(&email).await;

            match r {
                Ok(()) => println!("Email sent"),
                Err(err) => println!("Error {}", err),
            };

            // redirect to page
            HttpResponse::Found().header("Location", format!("/{}/password_email_sent", &lang)).finish()

        },
        Err(err) => {
            println!("Error: {}", err);
            HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
        }
    }
}

#[get("/{lang}/password_email_sent")]
pub async fn password_email_sent(
     web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let rendered = data.tmpl.render("authentication/password_email_sent.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/password_reset/{token}")]
pub async fn password_reset(
    web::Path((lang, token)): web::Path<(String, String)>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let result = PasswordResetToken::find_by_token(&token);

    match result {
        Ok(verified_token) => {
            let user = User::find_from_email(&verified_token.email_address).expect("Unable to load user from email");

            let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

            ctx.insert("user", &user);
            ctx.insert("token", &token);

            let rendered = data.tmpl.render("authentication/change_password.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            // token not valid return to login screen
            println!("Error: {}", &err);
            return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
        }
    };
}

#[post("/{lang}/password_reset/{token}")]
pub async fn password_reset_post(
    web::Path((lang, token)): web::Path<(String, String)>,
    _data: web::Data<AppData>,
    _req: HttpRequest, 
    form: web::Form<PasswordForm>,
    id: Identity,
) -> impl Responder {

    // validate form has data or re-load form
    if form.password.is_empty() || &token == "" {
        return HttpResponse::Found().header("Location", format!("/{}/login", &lang)).finish()
    };

    // update user
    let reset_token = PasswordResetToken::find_by_token(&token);

    match reset_token {
        Ok(reset_token) => {

            // check token is valid
            if chrono::Utc::now().naive_local() > reset_token.expires_on {
                return HttpResponse::Found().header("Location", format!("/{}/login", &lang)).finish()
            };

            let user = User::find_from_email(&reset_token.email_address).expect("Unable to load user from token email");

            let result = User::update_password(user.id, &form.password.trim().to_string());

            match result {
                Ok(user) => {
                    println!("User {} password updated", &user.user_name);

                    // delete token
                    PasswordResetToken::delete(reset_token.id).expect("Unable to delete token");

                    // log in user
                    id.remember(user.slug.to_owned());
                        
                    return HttpResponse::Found().header("Location", format!("/{}/user/{}", &lang, user.slug)).finish()
                },
                Err(err) => {
                    println!("Error: {}", err);
                    return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()

                }
            };
        },
        Err(err) => {
            println!("Error: {}", err);
            return HttpResponse::Found().header("Location", format!("/{}/log_in", &lang)).finish()
        }
    };
}