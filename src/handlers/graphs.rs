use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use serde_json::json;
use serde::{Serialize, Deserialize};
use petgraph::prelude::*;
use petgraph::dot::{Dot, Config};
use std::fmt;
use std::hash::Hash;

use num_bigint::{ToBigInt};
use bigdecimal::BigDecimal;

use crate::models::{NewPerson, Lens, Lenses, Node, Nodes, People};
use crate::database;

use crate::schema::{people, lenses, nodes};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct GNode {
    pub node_type: String,
    pub label: String,
    pub statements: Vec<String>,
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.label, self.node_type)
    }
}


#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>
) -> impl Responder {
        
    let people_vec = People::find_all().expect("Unable to load people");
    
    let lens_vec = Lenses::find_all().expect("Unable to load lenses");

    let node_vec = Nodes::find_all().expect("Unable to load nodes");

    let mut graph: Graph::<GNode, BigDecimal> = Graph::new();

    let mut people_index: Vec<NodeIndex> = Vec::new();
    let mut node_index: Vec<NodeIndex> = Vec::new();

    for p in people_vec {

        let ni = graph.add_node(GNode {
            node_type: String::from("Person"),
            label: format!("P-{}", p.id),
            statements: Vec::new(),
        });
        people_index.push(ni);
    };

    for n in node_vec {

        let ni = graph.add_node(GNode {
            node_type: String::from("Node"),
            label: format!("N-{}", &n.node_name),
            statements: Vec::new(),
        });

        node_index.push(ni);
    };

    for l in lens_vec {
        let ni = graph.add_node(GNode {
            node_type: String::from("Lens"),
            label: format!("L-{}", &l.id),
            statements: l.statements,
        });

        let _person_edge = graph.add_edge(ni, people_index[l.person_id as usize - 1], l.inclusivity.clone());
        let _node_edge = graph.add_edge(ni, node_index[l.node_id as usize - 1], l.inclusivity.clone());

        node_index.push(ni);
    };


    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("full_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/person_network_graph/{code}")]
pub async fn person_network_graph(
    web::Path(code): web::Path<String>,
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let person: People = people::table.filter(people::code.eq(code))
        .first(&conn)
        .expect("Unable to load person");
    
    let mut people_vec: Vec<People> = Vec::new();
    
    let zero_len: usize = 0;
    
    if &person.related_codes.len() > &zero_len {
        people_vec.push(person.clone());
        
        for c in &person.related_codes {
            people_vec.push(People::find_from_code(c).unwrap());
        }
    } else {
        people_vec.push(person);
    };
    
    // join lenses and nodes
    let node_lenses: Vec<(Lenses, Nodes)> = Lenses::belonging_to(&people_vec)
        .inner_join(nodes::table)
        .load::<(Lenses, Nodes)>(&conn)
        .expect("Error leading people");
    
    // group node_lenses by people
    let grouped = node_lenses.grouped_by(&people_vec);
    
    // structure result
    let result: Vec<(People, Vec<(Lenses, Nodes)>)> = people_vec
        .into_iter()
        .zip(grouped)
        .collect();

    let j = serde_json::to_string(&result).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("full_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/node_network_graph/{id}")]
pub async fn node_network_graph(
    web::Path(id): web::Path<i32>,
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let node: Nodes = nodes::table.filter(nodes::id.eq(id))
        .first(&conn)
        .expect("Unable to load person");
    
    let node_vec: Vec<Nodes> = vec![node];
        
    // join lenses and nodes
    let people_lenses: Vec<(Lenses, People)> = Lenses::belonging_to(&node_vec)
        .inner_join(people::table)
        .load::<(Lenses, People)>(&conn)
        .expect("Error leading people");
    
    // group node_lenses by people
    let grouped = people_lenses.grouped_by(&node_vec);
    
    // structure result
    let result: Vec<(Nodes, Vec<(Lenses, People)>)> = node_vec
        .into_iter()
        .zip(grouped)
        .collect();

    let j = serde_json::to_string(&result).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("full_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

