mod base;
mod api;
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
    use serde::Deserialize;
    use actix_web::{web, get, HttpResponse, HttpRequest, Responder};

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
}

pub use self::base::{
    raw_index,
    index,
    about,
};

pub use self::api::{
    api_base, 
    find_experience, 
    person_api,
    api_phrases,
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

pub use self::people::{person_page, delete_person, delete_person_post};

pub use self::nodes::{node_graph, node_page, community_node_page, community_node_graph};

pub use self::users::{user_index, user_page_handler, delete_user, delete_user_handler, edit_user, edit_user_post,
    admin_edit_user, admin_edit_user_post};

pub use self::community::{add_community, add_community_form_input, delete_community_form, delete_community,
    view_community, community_index, edit_community, edit_community_form_input, open_community_index};

pub use self::email::{email_person_info, send_community_email, EmailForm};

pub use self::authentication_handlers::{register_handler, register_form_input, registration_error, login_handler, login_form_input, logout,
    email_verification, resend_email_verification, verify_code, password_reset, password_reset_post, request_password_reset_post,
    password_email_sent, request_password_reset};
    
pub use self::utility::{DeleteForm, UrlParams, toggle_language, toggle_language_index, toggle_language_two, toggle_language_three};