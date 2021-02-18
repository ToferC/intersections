use std::env;
use std::sync::{Mutex, Arc};
use actix_web::{App, HttpServer, middleware, web};
use actix_session::{CookieSession};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web_static_files;
use tera::Tera;

use database;
use webapp::handlers; 
use webapp::models;
use webapp::AppData;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_rt::main] 
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let environment = env::var("ENVIRONMENT");
    let cookie_secret_key = env::var("COOKIE_SECRET").expect("Cant load cookie secret variable");

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

    let node_names = models::Nodes::find_all_linked_names().expect("Unable to load names");
    
    let x = Arc::new(Mutex::new(graph));
    let y = Arc::new(Mutex::new(node_names));
    
    println!("Serving on: {}:{}", &host, &port);
    
    HttpServer::new(move || {
        
        let mut tera = Tera::new(
            "templates/**/*").unwrap();
            
        tera.full_reload().expect("Error running auto reload with Tera");
            
        let x = Arc::clone(&x);
        let node_names = Arc::clone(&y);
        
        // load node_names for navbar population
            
        let generated = generate();

        App::new()
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
            .service(actix_web_static_files::ResourceFiles::new(
                "/static", generated,
            ))
            .data(AppData {
                tmpl: tera,
            })
            .app_data(web::Data::from(x))
            .app_data(web::Data::from(node_names))
            .wrap(CookieSession::signed(&[0; 32])
                .secure(false)
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&cookie_secret_key.as_bytes())
            .name("user-auth")
            .secure(false)))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
