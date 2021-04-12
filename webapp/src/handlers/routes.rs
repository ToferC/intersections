use actix_web::{web};

use crate::handlers::{
    // base
    index,

    // errors
    internal_server_error,
    not_found,
    
    // survey
    survey_intro,
    experience_form_handler, 
    handle_experience_form_input,
    add_handle_experience_form_input,

    // experience
    find_experience,

    // people
    person_page,
    delete_person,
    delete_person_post,

    // emails
    email_person_info,
    send_community_email,
    // nodes
    node_page,
    community_node_page,
    node_graph,
    community_node_graph,
    // API
    api_base,
    add_experience_form_handler,
    person_api,
    // graphs
    global_graph,
    person_graph,
    full_community_node_graph,

    // registration
    register_form_input,
    register_handler,
    registration_error,

    // email validation
    email_verification,
    verify_code,

    // password reset
    request_password_reset,
    request_password_reset_post,
    password_email_sent,
    password_reset,
    password_reset_post,

    // login
    login_handler,
    login_form_input,
    logout,

    // users
    user_index,
    user_page_handler,
    edit_user,
    edit_user_post,
    delete_user,
    delete_user_handler,

    // communities
    view_community,
    community_index,
    open_community_index,
    add_community,
    add_community_form_input,
    edit_community_form_input,
    delete_community,
    delete_community_form,
    edit_community,
    };

pub fn init_routes(config: &mut web::ServiceConfig) {
    // base
    config.service(index);
    config.service(find_experience);

    // errors
    config.service(internal_server_error);
    config.service(not_found);
    
    // survey
    config.service(survey_intro);
    config.service(experience_form_handler);
    config.service(handle_experience_form_input);
    config.service(add_experience_form_handler);
    config.service(add_handle_experience_form_input);
    
    // people
    config.service(person_page);
    config.service(delete_person);
    config.service(delete_person_post);

    // emails
    config.service(email_person_info);
    config.service(send_community_email);

    // nodes

    // node
    config.service(node_page);
    // community_node
    config.service(community_node_page);
    // node_graph
    config.service(node_graph);
    // community_node_graph
    config.service(community_node_graph);

    // api
    config.service(api_base);
    config.service(person_api);
    // graphs
    config.service(global_graph);
    config.service(person_graph);
    config.service(full_community_node_graph);

    // registration and validation
    config.service(register_handler);
    config.service(register_form_input);
    config.service(registration_error);
    config.service(email_verification);
    config.service(verify_code);

    // forgot password
    config.service(request_password_reset);
    config.service(request_password_reset_post);
    config.service(password_email_sent);
    config.service(password_reset);
    config.service(password_reset_post);

    // login and logout
    config.service(login_handler);
    config.service(login_form_input);
    config.service(logout);
    
    // users 
    config.service(user_page_handler);
    config.service(user_index);
    config.service(edit_user);
    config.service(edit_user_post);
    config.service(delete_user);
    config.service(delete_user_handler);

    // communities
    config.service(view_community);
    config.service(community_index);
    config.service(open_community_index);
    config.service(add_community);
    config.service(add_community_form_input);
    config.service(delete_community);
    config.service(delete_community_form);
    config.service(edit_community);
    config.service(edit_community_form_input);

}