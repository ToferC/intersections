use std::{sync::Mutex};

use actix_web::{web, get, HttpResponse, Responder};
use actix_identity::Identity;
use crate::{AppData, extract_identity_data};
use tera::{Context};

use std::collections::HashMap;

use crate::models::{Lenses, Communities, People};
use crate::handlers::{CytoGraph, generate_node_cyto_graph};

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
    node_names: web::Data<Mutex<Vec<String>>>,
    id: Identity,
) -> impl Responder {

    let g = graph.lock().unwrap().clone();
        
    let j = serde_json::to_string_pretty(&g).unwrap();

    drop(graph);
    
    let mut ctx = Context::new();

    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/full_node_graph")]
pub async fn full_node_graph(
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<String>>>,
    id:Identity,
) -> impl Responder {
        
    let lens_vec = Lenses::find_all().expect("Unable to load lenses");

    // create vec of bridge connections from people
    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();

    for l in &lens_vec {
        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
    };

    // now instead of building AggLens, we build the graph
    let graph = generate_node_cyto_graph(lens_vec, people_connections);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();

    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    ctx.insert("graph_data", &j);

    let title = "Node Network Graph";
    ctx.insert("title", title);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
    let rendered = data.tmpl.render("node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/community/node_graph/{community_code}")]
pub async fn community_node_graph(
    data: web::Data<AppData>,
    web::Path(community_code): web::Path<String>,
    node_names: web::Data<Mutex<Vec<String>>>,
    id:Identity,
) -> impl Responder {

    // Validate community
    let community_result = Communities::find_from_code(&community_code);

    match community_result {
        Ok(community) => {
            let lens_vec = Lenses::find_all().expect("Unable to load lenses");

            let community_people_ids = People::find_ids_from_community(community.id).expect("Unable to find community members");
        
            // create vec of bridge connections from people
            let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();
        
            for l in &lens_vec {
                if community_people_ids.contains(&l.person_id) {
                    people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
                }
            };
        
            // now instead of building AggLens, we build the graph
            let graph = generate_node_cyto_graph(lens_vec, people_connections);
        
            let j = serde_json::to_string_pretty(&graph).unwrap();
            
            let mut ctx = Context::new();
        
            let (session_user, role) = extract_identity_data(&id);
            ctx.insert("session_user", &session_user);
            ctx.insert("role", &role);
        
            ctx.insert("graph_data", &j);
        
            let title = "Node Network Graph";
            ctx.insert("title", title);
        
            // add node_names for navbar drop down
            ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
            
            let rendered = data.tmpl.render("node_network_graph.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::Found().header("Location","/").finish()
        },
    }      
}



