use std::sync::Mutex;

use actix_web::{web, get, HttpResponse, Responder};
use actix_session::{UserSession};
use actix_identity::Identity;
use crate::{AppData, extract_identity_data};
use tera::{Context};

use std::collections::HashMap;

use crate::models::{Nodes, Lenses};
use crate::handlers::{CytoGraph, generate_node_cyto_graph};

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
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

    let node_names = Nodes::find_all_linked_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/full_node_graph")]
pub async fn full_node_graph(
    data: web::Data<AppData>,
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

    let node_names = Nodes::find_all_linked_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    
    let rendered = data.tmpl.render("node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

