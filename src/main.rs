#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use std::env;
use actix_web::{App, HttpServer};
use tera::Tera;

mod models;
mod handlers;
mod schema;
mod database;
mod error_handler;

pub struct AppData {
    tmpl: Tera
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let environment = env::var("ENVIRONMENT");

    let environment = match environment {
        Ok(v) => v,
        Err(_) => String::from("test"),
    };

    let (host, port) = if environment == "production" {
        (env::var("HOST").unwrap(), env::var("PORT").unwrap())
    } else {
        (String::from("127.0.0.1"), String::from("8088"))
    };

    database::init();

    HttpServer::new(move || {

        let mut tera = Tera::new(
            "templates/**/*").unwrap();

        tera.full_reload().expect("Error running auto reload with Tera");

        App::new()
            .configure(handlers::init_routes)
            .data(AppData {tmpl: tera} )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
