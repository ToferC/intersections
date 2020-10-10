use actix_web::{web};

use crate::handlers::{
    lens_form_handler, 
    handle_lens_form_input,
    index,
    find_person,
    find_lens,
    api_base,
    add_lens_form_handler,
    add_handle_lens_form_input,
    find_person_from_code,
    person_api,
    full_network_graph,
    person_network_graph,
    node_network_graph,
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
    config.service(find_person_from_code);
    config.service(person_api);
    config.service(full_network_graph);
    config.service(person_network_graph);
    config.service(node_network_graph);
}