use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::hash::Hash;

use num_bigint::{ToBigInt};
use bigdecimal::{BigDecimal, ToPrimitive};

use crate::models::{Lenses, Node, Nodes, People};
use crate::database;

use crate::schema::{people, nodes};
use crate::handlers::{CytoGraph, CytoEdge, CytoNode, GNode, GEdge, generate_cyto_graph,
    generate_node_cyto_graph};

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>
) -> impl Responder {
        
    let people_vec = People::find_all().expect("Unable to load people");
    
    let lens_vec = Lenses::find_all().expect("Unable to load lenses");

    let node_vec = Nodes::find_all().expect("Unable to load nodes");

    let graph = generate_cyto_graph(people_vec, node_vec, lens_vec);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/node_network_graph/{label}")]
pub async fn node_network_graph(
    web::Path(label): web::Path<String>,
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let node: Nodes = nodes::table.filter(nodes::node_name.eq(label))
        .first(&conn)
        .expect("Unable to load person");
    
    let node_vec: Vec<Nodes> = vec![node];
        
    // join lenses and nodes
    let lens_vec: Vec<Lenses> = Lenses::belonging_to(&node_vec)
        .load::<Lenses>(&conn)
        .expect("Error leading people");
    
    let graph = generate_node_cyto_graph(node_vec, lens_vec);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);

    let title = "Node Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

