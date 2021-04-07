mod handlers;
mod survey_handlers;
mod routes;
mod graphs;
mod people_handlers;
mod node_handlers;
mod user_handlers;
mod community_handler;
mod email_handlers;
mod authentication_handlers;

mod utility {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct DeleteForm {
        pub verify: String,
    }
}

pub use self::handlers::{api_base, index,
    find_experience, person_api, f404};
    
pub use self::survey_handlers::{
    survey_intro,
    experience_form_handler, 
    handle_experience_form_input,
    add_experience_form_handler,
    add_handle_experience_form_input,
    RenderPerson,
};
pub use self::routes::init_routes;
pub use self::graphs::{global_graph, full_community_node_graph};
pub use self::people_handlers::{person_graph, person_page, delete_person, delete_person_post};
pub use self::node_handlers::{node_graph, node_page, community_node_page, community_node_graph};
pub use self::user_handlers::{user_index, user_page_handler, delete_user, delete_user_handler, edit_user, edit_user_post};
pub use self::community_handler::{add_community, add_community_form_input, delete_community_form, delete_community,
    view_community, community_index, edit_community, edit_community_form_input, open_community_index};
pub use self::email_handlers::{email_person_info, send_community_email, EmailForm};
pub use self::authentication_handlers::{register_handler, register_form_input, login_handler, login_form_input, logout,
    email_verification, verify_code, password_reset, password_reset_post, request_password_reset_post,
    password_email_sent, request_password_reset};
pub use self::utility::{DeleteForm};