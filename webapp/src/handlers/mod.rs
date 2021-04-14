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
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct DeleteForm {
        pub verify: String,
    }
}

pub use self::base::{
    api_base, 
    index,
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
    
pub use self::utility::{DeleteForm};