// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use std::sync::Mutex;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_session::{Session, UserSession};
use actix_identity::{Identity};
use tera::Context;
use serde::{Deserialize};

use crate::{AppData, extract_identity_data};
use crate::models::{User, verify, UserData, EmailVerification, InsertableVerification, Email};

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

    let rendered = data.tmpl.render("authentication/log_in.html", &ctx).unwrap();
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

    let rendered = data.tmpl.render("authentication/register.html", &ctx).unwrap();
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
    
    HttpResponse::Found().header("Location", String::from("/email_verification")).finish()
}

#[get("/email_verification")]
pub async fn email_verification(
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

    if session_user == "".to_string() && role != "admin".to_string() {
        // person signed in shouldn't be here
        return HttpResponse::Found().header("Location", "/").finish()
    };

    let user = User::find_from_slug(&session_user);

    match user {
        Ok(user) => {
            ctx.insert("user", &user);

            // send verification email
            let verification = EmailVerification::create(
                &InsertableVerification::new(&user.email)
            ).expect("Unable to create verification");

            let mut email_ctx = Context::new();
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

            // add node_names for navbar drop down
            ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
        
            let rendered = data.tmpl.render("authentication/email_verification.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("No user found: {}", err);
            return HttpResponse::Found().header("Location", "/").finish()
        },
    }
}

#[post("/verify_code")]
pub async fn verify_code(
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
        return HttpResponse::Found().header("Location", String::from("/email_verification")).finish()
    };

    // load user
    let mut user = User::find_from_slug(&session_user).expect("Unable to load user");

    let verification_code = EmailVerification::find_by_email(&user.email).expect("Unable to load email verification");

    // verify code entered vs code in email
    if form.code.trim() != verification_code.activation_code {
        // code doesn't match
        return HttpResponse::Found().header("Location", String::from("/email_verification")).finish()
    };
    
    // validate user
    user.validated = true;
    let user = User::update(user).expect("Unable to update user");

    // delete email_verification
    EmailVerification::delete(verification_code.id).expect("Unable to delete verification code");
    
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

