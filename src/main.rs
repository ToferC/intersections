#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use std::env;
use std::sync::{Mutex};
use actix_web::{App, HttpServer, middleware};
use tera::Tera;

mod models;
mod handlers;
mod schema;
mod database;
mod error_handler;

pub struct AppData {
    pub tmpl: Tera,
    pub graph: Mutex<handlers::CytoGraph>,
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

    println!("Loading data for graph");
    let people_vec = models::People::find_all().expect("Unable to load people");
    let node_vec = models::Nodes::find_all().expect("Unable to load nodes");
    let lens_vec = models::Lenses::find_all().expect("Unable to load lenses");

    // load master graph into app data
    println!("Generate graph representation");
    let graph: handlers::CytoGraph = handlers::generate_cyto_graph(people_vec, node_vec, lens_vec);

    println!("Serving on: {}:{}", &host, &port);

    HttpServer::new(move || {

        let mut tera = Tera::new(
            "templates/**/*").unwrap();

        tera.full_reload().expect("Error running auto reload with Tera");

        App::new()
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
            .data(AppData {
                tmpl: tera,
                graph: Mutex::new(graph.clone())} )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
