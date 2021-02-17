// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use actix_web::{HttpRequest, HttpResponse, Responder, dev::HttpResponseBuilder, get, post, web};
use actix_session::{Session, UserSession};
use tera::Context;
use serde::{Deserialize, Serialize};

use crate::{AppData, extract_session_data};
use crate::models::{User, SlimUser, LoggedUser, verify, UserData, Nodes};
use error_handler::error_handler::CustomError;

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

#[get("/user_index")]
pub async fn user_index(
    data: web::Data<AppData>,
    req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    // validate if user is admin else redirect

    let session = req.get_session();

    let (session_user, role) = extract_session_data(&session);
    
    if role != "admin".to_string() {
        HttpResponse::Found().header("Location", "/log_in").finish()
    } else {

        ctx.insert("session_user", &session_user);
        ctx.insert("role", &role);

        let node_vec = Nodes::find_all_linked_names().expect("Unable to load nodes");
        ctx.insert("node_names", &node_vec);

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


#[get("/user/{user_name}")]
pub async fn user_page_handler(
    web::Path(user_name): web::Path<String>,
    data: web::Data<AppData>,
    req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    // validate if user == user_name  or admin else redirect

    let session = req.get_session();

    let (session_user, role) = extract_session_data(&session);
    
    if session_user != user_name.clone() || role != "admin".to_string() {
        HttpResponse::Found().header("Location", "/log_in").finish()
    } else {

        ctx.insert("session_user", &session_user);
        ctx.insert("role", &role);

        let node_vec = Nodes::find_all_linked_names().expect("Unable to load nodes");
        ctx.insert("node_names", &node_vec);
    
    
        let user = User::find_from_user_name(&user_name).expect("Could not load user");
    
        ctx.insert("user", &user);
    
        let rendered = data.tmpl.render("user_page.html", &ctx).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/log_in")]
pub async fn login_handler(data: web::Data<AppData>, req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    // Get session data and add to context
    let session = req.get_session();
    let (session_user, role) = extract_session_data(&session);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let node_vec = Nodes::find_all_linked_names().expect("Unable to load nodes");
    ctx.insert("node_names", &node_vec);

    let rendered = data.tmpl.render("log_in.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/log_in")]
pub async fn login_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<LoginForm>,
    session: Session,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() {
        println!("Form is empty");
        return HttpResponse::Found().header("Location", String::from("/log_in")).finish()
    };
    
    let user = User::find_from_email(&form.email);

    match user {
        Ok(u) => {
            let user = u;
            println!("{:?}", &form);
        
            if verify(&user, &form.password) {
                println!("Verified");

                session.set("role", user.role.to_owned()).expect("Unable to set role cookie");
                session.set("session_user", user.user_name.to_owned()).expect("Unable to set user name");
        
                return HttpResponse::Found().header("Location", format!("/user/{}", user.user_name)).finish()
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
pub async fn register_handler(data: web::Data<AppData>, req:HttpRequest) -> impl Responder {
    
    let mut ctx = Context::new();

    // Get session data and add to context
    let session = req.get_session();
    let (session_user, role) = extract_session_data(&session);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let node_vec = Nodes::find_all_linked_names().expect("Unable to load nodes");
    ctx.insert("node_names", &node_vec);

    let rendered = data.tmpl.render("register.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/register")]
pub async fn register_form_input(
    _data: web::Data<AppData>,
    req: HttpRequest, 
    form: web::Form<RegisterForm>,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.email.is_empty() || form.password.is_empty() || form.user_name.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/register")).finish()
    };

    // create user

    let user_data = UserData {
        email: form.email.to_owned(),
        user_name: form.user_name.to_owned(),
        password: form.password.to_owned(),
        role: "user".to_owned(),
    };

    let user = User::create(user_data).expect("Unable to load user.");

    let session = req.get_session();

    session.set("role", user.role.to_owned()).expect("Unable to set role cookie");
    session.set("session_user", user.user_name.to_owned()).expect("Unable to set user name");
    
    HttpResponse::Found().header("Location", format!("/user/{}", user.user_name)).finish()
}

#[post("/log_out")]
pub async fn logout(
    _data: web::Data<AppData>,
    req: HttpRequest,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let session = req.get_session();

    session.clear();

    HttpResponse::Found().header("Location", "/").finish()
}