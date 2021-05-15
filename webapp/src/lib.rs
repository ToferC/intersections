#[macro_use]
extern crate diesel;
use std::sync::Mutex;

use actix_web::{App, web};
use tera::{Tera, Context};
use actix_session::Session;
use actix_identity::Identity;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use sendgrid::SGClient;

pub mod models;
pub mod handlers;
pub mod schema;

#[derive(Clone, Debug)]
pub struct AppData {
    pub tmpl: Tera,
    pub mail_client: SGClient,
}

pub fn extract_session_data(session: &Session) -> (String, String) {

    let role_data = session.get::<String>("role").expect("Unable to get role from cookie");

    let role = match role_data {
        Some(r) => r,
        None => "".to_string(),
    };

    let user_data = session.get::<String>("user_name").expect("Unable to get user_name from cookie");

    let session_user = match user_data {
        Some(u) => u,
        None => "".to_string(),
    };

    println!("{}-{}", &session_user, &role);

    (session_user, role)
}

pub fn extract_identity_data(id: &Identity) -> (String, String) {

    let id_data = id.identity();

    let session_user = match id_data {
        Some(u) => u,
        None => "".to_string(),
    };

    let user = models::User::find_slim_from_slug(&session_user);

    let role = match user {
        Ok(u) => u.role,
        _ => "".to_string()
    };

    println!("{}-{}", &session_user, &role);

    (session_user, role)
}

/// Generate context, session_user and role from id and node_names
pub fn generate_basic_context(
        id: Identity,
        lang: &str,
        path: &str,
        node_names: web::Data<Mutex<Vec<(String, String)>>>) -> (Context, String, String, String
    ) 
{    
    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let validated_lang = match lang {
        "fr" => "fr",
        "en" => "en",
        _ => "en",
    };

    ctx.insert("lang", &validated_lang);
    ctx.insert("path", &path);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    (ctx, session_user, role, lang.to_owned())
}

/// Generate context, session_user and role from id and lang
pub fn generate_email_context(
    id: Identity,
    lang: &str,
    path: &str,) -> (Context, String, String, String) 
{    
let mut ctx = Context::new();

// Get session data and add to context
let (session_user, role) = extract_identity_data(&id);
ctx.insert("session_user", &session_user);
ctx.insert("role", &role);

let validated_lang = match lang {
    "fr" => "fr",
    "en" => "en",
    _ => "en",
};

ctx.insert("lang", &validated_lang);
ctx.insert("path", &path);

(ctx, session_user, role, lang.to_owned())
}

pub fn generate_unique_code(mut characters: usize, dashes: bool) -> String {

    if characters > 64 {
        characters = 64;
    };

    let mut rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(characters)
        .collect();

    if dashes {
        for i in 0..rand_string.len() + rand_string.len() / 4 {
            if i > 2 && i % 4 == 0 {
                rand_string.insert(i, '-');
            }
        }
    };

    rand_string
}

use background_jobs::{Backoff, Job, MaxRetries};
use futures::future::{ok, Ready};
use serde::{Serialize, Deserialize};
use models::{RawExperience};

#[derive(Clone, Debug)]
pub struct AppState {
    pub app_name: String,
}

impl AppState {
    pub fn new(app_name: &str) -> Self {
        AppState {
            app_name: app_name.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslateJob {
    pub experience: RawExperience,
    pub lang: String,
}

impl TranslateJob {
    pub fn new(experience: &RawExperience, lang: &str) -> Self {
        TranslateJob {
            experience: experience.clone(),
            lang: lang.to_owned(),
        }
    }
}

impl Job for TranslateJob {
    type State = AppState;
    type Future = Ready<Result<(), anyhow::Error>>;
    const NAME: &'static str = "TranslateJob";
    const QUEUE: &'static str = "JobQueue";
    const MAX_RETRIES: MaxRetries = MaxRetries::Count(5);
    const BACKOFF: Backoff = Backoff::Exponential(3);

    fn run(mut self, _: Self::State, ) -> Self::Future {

        let _result = self.experience.translate_experience_phrases(self.lang.as_str());

        ok(())    
}

    const TIMEOUT: i64 = 15_000;
}