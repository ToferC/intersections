use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_session::{UserSession};
use crate::{AppData, extract_session_data};
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use std::collections::HashMap;

use crate::models::{Lenses, Nodes};
use database;
use crate::handlers::{AggLens, generate_node_cyto_graph};

use crate::schema::{nodes, lenses};

#[get("/node/{label}")]
pub async fn node_page(
    web::Path(label): web::Path<String>, 
    data: web::Data<AppData>, 
    req:HttpRequest,
) -> impl Responder {
    let mut ctx = Context::new();

    // Get session data and add to context
    let session = req.get_session();
    let (session_user, role) = extract_session_data(&session);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let conn = database::connection().expect("Unable to connect to db");
    
    let node: Nodes = nodes::table.filter(nodes::node_name.eq(label))
        .first(&conn)
        .expect("Unable to load node");
    
    // get connected nodes via people with lense connections to our prime node

    let lens_vec: Vec<Lenses> = Lenses::belonging_to(&node)
        .load::<Lenses>(&conn)
        .expect("Error leading connected lenses");

    let mut people_id_vec: Vec<i32> = Vec::new();
    let mut node_id_vec: Vec<i32> = Vec::new();

    for l in &lens_vec {
        people_id_vec.push(l.person_id);
        node_id_vec.push(l.node_id);
    };

    people_id_vec.sort();
    people_id_vec.dedup();


    // add lenses for the people connected by node
    let connected_lenses = lenses::table.filter(lenses::person_id.eq_any(&people_id_vec))
        .load::<Lenses>(&conn)
        .expect("Unable to load lenses");
    
    for l in &connected_lenses {
        node_id_vec.push(l.node_id);
    };

    node_id_vec.sort();
    node_id_vec.dedup();
    
    println!("nodes: {:?}, people: {:?}", &node_id_vec, &people_id_vec);
    
    let mut aggregate_lenses: Vec<AggLens> = Vec::new();

    for i in node_id_vec {
        let mut temp_lens_vec: Vec<Lenses> = Vec::new();

        for l in &connected_lenses {

            if i == l.node_id && i != node.id {
                temp_lens_vec.push( Lenses {
                    id: l.id,
                    node_name: l.node_name.to_owned(),
                    node_domain: l.node_domain.to_owned(),
                    person_id: l.person_id,
                    node_id: l.node_id,
                    date_created: l.date_created,
                    statements: l.statements.to_owned(),
                    inclusivity: l.inclusivity.clone(),
                });
            }
            // count people associated to multiple similar nodes
            // show connections across the nodes and lenses
        };

        if temp_lens_vec.len() > 0 {
            let agg_lenses = AggLens::from(temp_lens_vec);
            aggregate_lenses.push(agg_lenses);
        }
    };

    aggregate_lenses.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
    aggregate_lenses.dedup();

    // Aggregate info from lenses related to the prime node
    let node_lens = AggLens::from(lens_vec);

    ctx.insert("node", &node);
    
    ctx.insert("node_lens", &node_lens);

    ctx.insert("other_lenses", &aggregate_lenses);

    let node_names = Nodes::find_all_linked_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);

    let rendered = data.tmpl.render("node.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/node_network_graph/{label}")]
pub async fn node_network_graph(
    // Rework this as a connected node graph
    web::Path(label): web::Path<String>,
    data: web::Data<AppData>,
    req: HttpRequest,
) -> impl Responder {
    
    let mut ctx = Context::new();

    // Get session data and add to context
    let session = req.get_session();
    let (session_user, role) = extract_session_data(&session);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let conn = database::connection().expect("Unable to connect to db");
    
    let node: Nodes = nodes::table.filter(nodes::node_name.eq(label))
        .first(&conn)
        .expect("Unable to load node");
    
    // get connected nodes via people with lense connections to our prime node

    let mut lens_vec: Vec<Lenses> = Lenses::belonging_to(&node)
        .load::<Lenses>(&conn)
        .expect("Error leading connected lenses");

    let mut people_id_vec: Vec<i32> = Vec::new();
    let mut node_id_vec: Vec<i32> = Vec::new();

    // create vec of bridge connections from people
    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();

    for l in &lens_vec {
        people_id_vec.push(l.person_id);
        node_id_vec.push(l.node_id);
    };


    people_id_vec.sort();
    people_id_vec.dedup();

    // add lenses for the people connected by node
    let mut connected_lenses = lenses::table.filter(lenses::person_id.eq_any(&people_id_vec))
        .load::<Lenses>(&conn)
        .expect("Unable to load lenses");
    
    lens_vec.append(&mut connected_lenses);

    for l in &lens_vec {
        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.node_name.to_owned()); 
    };

    println!("{:?}", &people_connections);

    for l in &connected_lenses {
        node_id_vec.push(l.node_id);
    };

    // now instead of building AggLens, we build the graph
    let graph = generate_node_cyto_graph(lens_vec, people_connections);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    ctx.insert("graph_data", &j);

    let title = "Node Network Graph";
    ctx.insert("title", title);

    let node_names = Nodes::find_all_linked_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    
    let rendered = data.tmpl.render("node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}