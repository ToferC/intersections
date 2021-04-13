use std::sync::Mutex;
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};

pub async fn f404(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>, 
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (mut ctx, _, _) = generate_basic_context(id, node_names);

    let uri_path = req.uri().path();
    ctx.insert("path", &uri_path);

    let rendered = data.tmpl.render("errors/404.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/not_found")]
pub async fn not_found(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>, 
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _) = generate_basic_context(id, node_names);

    let rendered = data.tmpl.render("errors/not_found.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/internal_server_error")]
pub async fn internal_server_error(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>, 
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _) = generate_basic_context(id, node_names);

    let rendered = data.tmpl.render("errors/internal_server_error.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/not_authorized")]
pub async fn not_authorized(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>, 
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _) = generate_basic_context(id, node_names);

    let rendered = data.tmpl.render("errors/not_authorized.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}