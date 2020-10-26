use std::sync::Mutex;

use actix_web::{web, get, HttpResponse, Responder};
use crate::AppData;
use tera::{Context};

use crate::models::{Nodes};
use crate::handlers::CytoGraph;

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
) -> impl Responder {

    let g = graph.lock().unwrap().clone();
        
    let j = serde_json::to_string_pretty(&g).unwrap();

    drop(graph);
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);

    let node_names = Nodes::find_all_linked_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

