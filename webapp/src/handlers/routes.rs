use actix_web::{test::config, web};

use crate::handlers::{
    // forms
    lens_form_handler, 
    handle_lens_form_input,
    add_handle_lens_form_input,
    // pages
    index,
    survey_intro,
    person_page,
    node_page,
    find_lens,
    // API
    api_base,
    add_lens_form_handler,
    person_api,
    // graphs
    full_network_graph,
    full_node_graph,
    node_network_graph,
    person_graph,
    // registration
    register_form_input,
    register_handler,
    // login
    login_handler,
    login_form_input,
    logout,
    // users
    user_index,
    user_page_handler,
    // delete users
    delete_user,
    delete_user_handler,
    };

pub fn init_routes(config: &mut web::ServiceConfig) {
    // pages
    config.service(index);
    config.service(survey_intro);
    config.service(person_page);
    config.service(node_page);
    config.service(find_lens);
    // api
    config.service(api_base);
    config.service(person_api);
    // forms
    config.service(lens_form_handler);
    config.service(handle_lens_form_input);
    config.service(add_lens_form_handler);
    config.service(add_handle_lens_form_input);
    // graphs
    config.service(full_network_graph);
    config.service(full_node_graph);
    config.service(person_graph);
    config.service(node_network_graph);
    // users 
    config.service(register_handler);
    config.service(register_form_input);
    config.service(login_handler);
    config.service(login_form_input);
    config.service(logout);
    config.service(user_page_handler);
    config.service(user_index);
    config.service(delete_user);
    config.service(delete_user_handler);
}