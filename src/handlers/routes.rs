use actix_web::{web};

use crate::handlers::{
    // forms
    lens_form_handler, 
    handle_lens_form_input,
    add_handle_lens_form_input,
    // index
    index,
    find_person,
    find_lens,
    // API
    api_base,
    add_lens_form_handler,
    person_api,
    // graphs
    full_network_graph,
    node_network_graph,
    person_graph,
    };

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(index);
    config.service(find_person);
    config.service(find_lens);
    config.service(api_base);
    config.service(lens_form_handler);
    config.service(handle_lens_form_input);
    config.service(add_lens_form_handler);
    config.service(add_handle_lens_form_input);
    config.service(person_api);
    config.service(full_network_graph);
    config.service(person_graph);
    config.service(node_network_graph);
}