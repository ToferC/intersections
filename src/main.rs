extern crate diesel;

use std::env;
use actix_web::{App, HttpServer, web, middleware};
use tera::Tera;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod handlers;

use models::{Domain, Lens, LivedStatement};

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

    /*
    // create database connection pool
    // Diesel Postgres
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    */

    HttpServer::new(move || {

        let tera = Tera::new(
            "templates/**/*").unwrap();

        App::new()
            .data(AppData {tmpl: tera} )
            .service(handlers::index)
            .service(handlers::handle_lenses_form)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
