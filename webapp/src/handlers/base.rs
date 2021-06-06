use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};


#[get("/")]
pub async fn raw_index(
    _data: web::Data<AppData>,
    _req:HttpRequest,
) -> impl Responder {

    // Redirect if somone is getting the index with no language param
    return HttpResponse::Found().header("Location", "/en").finish()
}

#[get("/{lang}")]
pub async fn index(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());
    
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/about")]
pub async fn about(
    data: web::Data<AppData>,
    web::Path(lang): web::Path<String>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let rendered = data.tmpl.render("about.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}