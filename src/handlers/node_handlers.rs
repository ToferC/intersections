use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use serde::Serialize;

use bigdecimal::{ToPrimitive};
use std::collections::BTreeMap;

use crate::models::{Lenses, Nodes, People};
use crate::database;
use crate::handlers::{generate_cyto_graph, generate_node_cyto_graph, RenderPerson,
    AggLens};

use crate::schema::{people, nodes, lenses};

#[derive(Serialize, Debug)]
pub struct AggregateNodes {
    pub name: String,
    pub domain: String,
    pub count: u32,
    pub mean_inclusivity: f32,
    pub frequency_distribution: BTreeMap<String, u32>,
}

impl AggregateNodes {
    pub fn from(lenses: Vec<Lenses>) -> AggregateNodes {
        let name = &lenses[0].node_name;
        let domain = &lenses[0].node_domain;

        let mut inclusivity: f32 = 0.0;
        let mut counts = BTreeMap::new();

        for l in &lenses {
            inclusivity += l.inclusivity.to_f32().expect("Unable to convert bigdecimal");

            for s in &l.statements {
                *counts.entry(s.to_owned()).or_insert(0) += 1;
            };
        };

        let count = lenses.len() as u32;

        AggregateNodes {
            name: name.to_owned(),
            domain: domain.to_owned(),
            count: count,
            mean_inclusivity: inclusivity / count as f32,
            frequency_distribution: counts,
        }
    }
}

#[get("/node/{label}")]
pub async fn node_page(
    web::Path(label): web::Path<String>, 
    data: web::Data<AppData>, 
    _req:HttpRequest
) -> impl Responder {
    let mut ctx = Context::new(); 

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

    println!("nodes: {:?}, people: {:?}", &node_id_vec, &people_id_vec);

    // add lenses for the people connected by node
    let connected_lenses = lenses::table.filter(lenses::person_id.eq_any(people_id_vec))
        .load::<Lenses>(&conn)
        .expect("Unable to load lenses");

    
    // Something wrong with node id's here
    for l in &connected_lenses {
        node_id_vec.push(l.node_id);
    };

    node_id_vec.sort();
    node_id_vec.dedup();
    
    let mut aggregate_lenses: Vec<AggLens> = Vec::new();

    for i in node_id_vec {
        let mut temp_lens_vec: Vec<Lenses> = Vec::new();

        for l in &connected_lenses {
            // some kind of issue here
            if i == l.node_id {
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

        let agg_lenses = AggLens::from(temp_lens_vec);
        aggregate_lenses.push(agg_lenses);
    };

    // Aggregate info from lenses related to the prime node
    let node_lens = AggLens::from(lens_vec);

    ctx.insert("node", &node);
    
    ctx.insert("node_lens", &node_lens);

    ctx.insert("other_lenses", &aggregate_lenses);

    let rendered = data.tmpl.render("node.html", &ctx).unwrap();
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