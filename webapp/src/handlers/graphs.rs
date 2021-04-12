use std::{sync::Mutex};

use actix_web::{web, get, HttpResponse, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};

use std::collections::HashMap;

use crate::models::{Experiences, Communities, People, User};
use crate::models::{CytoGraph, generate_node_cyto_graph};

#[get("/data_global_graph")]
pub async fn data_global_graph(
    // no longer using appdata to hold graph
    // this function is a placeholder in case we go back here
    data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
) -> impl Responder {

    let g = graph.lock().unwrap().clone();
        
    let j = serde_json::to_string_pretty(&g).unwrap();

    drop(graph);
    
    let (mut ctx, _session_user, _role) = generate_basic_context(id, node_names);

    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/global_graph")]
pub async fn global_graph(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id:Identity,
) -> impl Responder {
        
    let experience_vec = Experiences::find_all_real().expect("Unable to load experiences");

    // create vec of bridge connections from people
    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();

    for l in &experience_vec {
        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
    };

    // now instead of building AggExperience, we build the graph
    let graph = generate_node_cyto_graph(experience_vec, people_connections, None);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let (mut ctx, _session_user, _role) = generate_basic_context(id, node_names);

    ctx.insert("graph_data", &j);

    let title = "Global Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/full_community_graph/{community_slug}")]
pub async fn full_community_node_graph(
    data: web::Data<AppData>,
    web::Path(community_slug): web::Path<String>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id:Identity,
) -> impl Responder {

    // Validate community
    let community_result = Communities::find_from_slug(&community_slug);

    match community_result {
        Ok(community) => {
            
            let (mut ctx, session_user, _role) = generate_basic_context(id, node_names);

            let session_user_id = match User::find_id_from_slug(&session_user) {
                Ok(id) => id,
                Err(_e) => 9999,
            };

            // validate if user can view community graph
            if community.open || session_user_id == community.user_id {
                
                let experience_vec = Experiences::find_all().expect("Unable to load experiences");
    
                let community_people_ids = People::find_ids_from_community(community.id).expect("Unable to find community members");
            
                // create vec of bridge connections from people
                let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();
            
                for l in &experience_vec {
                    if community_people_ids.contains(&l.person_id) {
                        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
                    }
                };
            
                // now instead of building AggExperience, we build the graph
                let graph = generate_node_cyto_graph(experience_vec, people_connections, Some(community.slug));
            
                let j = serde_json::to_string_pretty(&graph).unwrap();
            
                ctx.insert("graph_data", &j);
            
                let title = format!("{} Community Node Graph", &community.tag);
                ctx.insert("title", &title);
                
                let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
                HttpResponse::Ok().body(rendered)
            } else {
                // user not validated to see community
                println!("User not validated to see community");
                return HttpResponse::Found().header("Location","/").finish()

            }
        },
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::Found().header("Location","/").finish()
        },
    }      
}



