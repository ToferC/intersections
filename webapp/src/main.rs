use std::env;
use actix_web::{App, HttpServer, middleware, web, guard};
use actix_web::cookie::Key;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web_static_files;
use tera::Tera;
use tera_text_filters::snake_case;
use sendgrid::SGClient;

use database;
use webapp::handlers; 
use webapp::AppData;

use fluent_templates::{FluentLoader, static_loader};
// https://lib.rs/crates/fluent-templates

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

static_loader! {
    static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[actix_web::main] 
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    
    let environment = env::var("ENVIRONMENT");

    let cookie_secret = env::var("COOKIE_SECRET_KEY").expect("Unable to find secret key");

    let cookie_secret_key: Key = Key::from(&cookie_secret.as_bytes());

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

    // SendGrid email API
    let sendgrid_var = env::var("SENDGRID_API_KEY");
    let sendgrid_key: String;

    match sendgrid_var {
        Ok(key) => sendgrid_key = key,
        Err(err) => panic!("Must supply API key in env variables to use: {}", err),
    };

    // to prepopulate database for test instance run databaseutils

    /*
    println!("Loading data for graph");
    let experience_vec = models::Experiences::find_all_real().expect("Unable to load experience");
    
    // create vec of bridge connections from people
    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();

    for l in &experience_vec {
        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
    };

    // load master graph into app data
    println!("Generate graph representation");
    let graph: models::CytoGraph = models::generate_node_cyto_graph(experience_vec, people_connections, None);
    let x = Arc::new(Mutex::new(graph));
    */

    /*
    Localization
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_languages(&Localizations, &[loader.fallback_language()])
        .unwrap();
     */ 
    
    println!("Serving on: {}:{}", &host, &port);
    
    HttpServer::new(move || {
        
        // templating
        let mut tera = Tera::new(
            "templates/**/*").unwrap();
        
        tera.register_filter("snake_case", snake_case);
        tera.full_reload().expect("Error running auto reload with Tera");
        tera.register_function("fluent", FluentLoader::new(&*LOCALES));

        // mail client
        let sg = SGClient::new(sendgrid_key.clone());
            
        let generated = generate();

        let data = web::Data::new(AppData {
            tmpl: tera,
            mail_client: sg,
        });

        App::new()
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
            .app_data(data.clone())
            .service(actix_web_static_files::ResourceFiles::new(
                "/static", generated,
            ))
            .default_service(
                web::route()
                    .guard(guard::Not(guard::Get()))
                    .to(handlers::f404),
            )
            // .app_data(web::Data::from(graph))
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(), cookie_secret_key.clone())
                    .cookie_secure(false)
                    .build()
                )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
