use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use tera::Context;

use crate::AppData;
use crate::models::{Lens};

#[post("/form_1")]
pub async fn handle_lenses_form(data: web::Data<AppData>, req: HttpRequest, params: web::Form<Lens>) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let ctx = Context::new(); 

    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(rendered)
}