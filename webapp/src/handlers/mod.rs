mod base;
mod surveys;
mod routes;
mod graphs;
mod people;
mod nodes;
mod users;
mod community;
mod email;
mod authentication_handlers;
mod errors;

mod utility {
    use error_handler::error_handler;
    use serde::Deserialize;
    use actix_web::{web, get, HttpResponse, HttpRequest, Responder};

    use crate::models::{Phrases, InsertablePhrase, RawExperience};

    use libretranslate::{translate, Language};


    #[derive(Deserialize, Debug)]
    pub struct DeleteForm {
        pub verify: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct UrlParams {
        pub lang: Option<String>,
    }

    #[get("/toggle_language/{lang}")]
    pub async fn toggle_language_index(
        web::Path(lang): web::Path<String>,
    ) -> impl Responder {

        let new_lang = match lang.as_str() {
            "fr" => "en",
            "en" => "fr",
            _ => "en",
        };

        println!("New lang: {}", &new_lang);

        HttpResponse::Found()
            .header("Accept-Language", new_lang)
            .header("Location", format!("/{}", &new_lang))
            .finish()
    }

    #[get("/toggle_language/{lang}/{url}")]
    pub async fn toggle_language(
        web::Path((lang, url)): web::Path<(String, String)>,
        _req: HttpRequest,
    ) -> impl Responder {
        println!("url: {}", &url);

        let new_lang = if lang.as_str() == "en" {
            "fr"
        } else {
            "en"
        };

        println!("New lang: {}", &new_lang);

        // Remove leading character "/"
        let cleaned_url: &str = url.split("/").into_iter().last().expect("Unable to find url");

        HttpResponse::Found()
            .header("Location", format!("/{}/{}", &new_lang, &cleaned_url))
            .finish()
    }

    #[get("/toggle_language/{lang}/{url}/{url2}")]
    pub async fn toggle_language_two(
        web::Path((lang, url, url2)): web::Path<(String, String, String)>,
        _req: HttpRequest,
    ) -> impl Responder {
        println!("url: {}/{}", &url, &url2);

        let new_lang = if lang.as_str() == "en" {
            "fr"
        } else {
            "en"
        };

        println!("New lang: {}", &new_lang);

        HttpResponse::Found()
            .header("Location", format!("/{}/{}/{}", &new_lang, &url, &url2))
            .finish()
    }

    #[get("/toggle_language/{lang}/{url}/{url2}/{url3}")]
    pub async fn toggle_language_three(
        web::Path((lang, url, url2, url3)): web::Path<(String, String, String, String)>,
        _req: HttpRequest,
    ) -> impl Responder {
        println!("url: {}/{}/{}", &url, &url2, &url3);

        let new_lang = if lang.as_str() == "en" {
            "fr"
        } else {
            "en"
        };

        println!("New lang: {}", &new_lang);

        HttpResponse::Found()
            .header("Location", format!("/{}/{}/{}/{}", &new_lang, &url, &url2, &url3))
            .finish()
    }

    pub async fn generate_experience_phrases(lang: &str, ex: RawExperience) -> Result<Vec<i32>, error_handler::CustomError> {
        // Translates a complete experience including node name and statements
        // Returns a String that is meant to be split on "\n."

        let mut translate_strings: Vec<String> = Vec::new();

        translate_strings.push(ex.node_name);
        
        for s in &ex.statements {
            translate_strings.push(format!("{}.\n", &s));
        };

        let mut source: Language = Language::English;
        let mut target = Language::French;

        let translate_lang = match &lang {
            &"en" => {
                source = Language::English;
                target = Language::French;
                "fr".to_string()
            },
            &"fr" => {
                source = Language::French;
                target = Language::English;
                "en".to_string()
            },
            _ => {
                source = Language::English;
                target = Language::French;
                "fr".to_string()
            },
        };

        let source = Language::English;

        let input = translate_strings.concat();

        let data = translate(source, target, input)
            .await
            .unwrap();

        let input = data.input.split(".\n");
        let output = data.output.split(".\n");

        let mut phrase_ids = Vec::new();

        for (i, o) in input.into_iter().zip(output) {

            let phrase = InsertablePhrase::new(lang, i.to_lowercase().trim().replace("/",""));

            let phrase = Phrases::create(&phrase).expect("Unable to create phrase");

            let trans = Phrases {
                id: phrase.id,
                lang: translate_lang.to_owned(),
                text: o.to_lowercase().trim().replace("/",""),
            };

            let translation = Phrases::add_translation(trans).expect("Unable to add translation phrase");
            
            phrase_ids.push(phrase.id);
        };

        Ok(phrase_ids)
    }
}

pub use self::base::{
    api_base, 
    raw_index,
    index,
    about,
    find_experience, 
    person_api, 
};

pub use self::errors::{
    f404,
    not_found,
    internal_server_error,
    not_authorized,
};
    
pub use self::surveys::{
    survey_intro,
    experience_form_handler, 
    handle_experience_form_input,
    add_experience_form_handler,
    add_handle_experience_form_input,
    RenderPerson,
};
pub use self::routes::init_routes;

pub use self::graphs::{global_graph, full_community_node_graph};

pub use self::people::{person_graph, person_page, delete_person, delete_person_post};

pub use self::nodes::{node_graph, node_page, community_node_page, community_node_graph};

pub use self::users::{user_index, user_page_handler, delete_user, delete_user_handler, edit_user, edit_user_post,
    admin_edit_user, admin_edit_user_post};

pub use self::community::{add_community, add_community_form_input, delete_community_form, delete_community,
    view_community, community_index, edit_community, edit_community_form_input, open_community_index};

pub use self::email::{email_person_info, send_community_email, EmailForm};

pub use self::authentication_handlers::{register_handler, register_form_input, registration_error, login_handler, login_form_input, logout,
    email_verification, verify_code, password_reset, password_reset_post, request_password_reset_post,
    password_email_sent, request_password_reset};
    
pub use self::utility::{DeleteForm, UrlParams, toggle_language, toggle_language_index, toggle_language_two, toggle_language_three};