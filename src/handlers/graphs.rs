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

#[derive(Serialize, Deserialize, Debug)]
pub struct CytoGraph {
    nodes: Vec<CytoNode>,
    edges: Vec<CytoEdge>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CytoNode {
    data: GNode,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CytoEdge {
    data: GEdge,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GEdge {
    id: String,
    source: String,
    target: String,
    weight: f32,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct GNode {
    pub id: String,
    pub node_type: String,
    pub text: Vec<String>,
    pub shape: String,
    pub size: i32,
    pub color: String,
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.id, self.node_type)
    }
}


#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>
) -> impl Responder {
        
    let people_vec = People::find_all().expect("Unable to load people");
    
    let lens_vec = Lenses::find_all().expect("Unable to load lenses");

    let node_vec = Nodes::find_all().expect("Unable to load nodes");

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    for p in people_vec {

        let ni = GNode {
            id: format!("P-{}", p.id),
            node_type: String::from("Person"),
            text: vec![format!("{}", p.date_created)],
            shape: String::from("ellipse"),
            size: 25,
            color: String::from("orange"),
        };

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for n in node_vec {

        let ni = GNode {
            id: format!("{}", &n.node_name),
            node_type: String::from("Node"),
            text: vec![n.domain_token],
            shape: String::from("triangle"),
            size: 25,
            color: String::from("blue"),
        };

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for l in lens_vec {
        let ni = GNode {
            id: format!("L-{}", &l.id),
            node_type: String::from("Lens"),
            text: l.statements,
            shape: String::from("square"),
            size: 25,
            color: String::from("green"),
        };

        let person_edge = GEdge {
            id: format!("L{}-P{}", &l.id, &l.person_id),
            source: format!("L-{}", &l.id),
            target: format!("P-{}", &l.person_id),
            weight: l.inclusivity.to_f32().unwrap(),
        };

        let node_edge = GEdge {
            id: format!("L{}-{}", &l.id, &l.node_name),
            source: format!("L-{}", &l.id),
            target: format!("{}", &l.node_name),
            weight: l.inclusivity.to_f32().unwrap(),
        };

        cyto_node_array.push(CytoNode {
            data: ni,
        });

        cyto_edge_array.push(CytoEdge {
            data: person_edge,
        });

        cyto_edge_array.push(CytoEdge {
            data: node_edge,
        });
    };

    let graph: CytoGraph = CytoGraph {
        nodes: cyto_node_array,
        edges: cyto_edge_array,
    };

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
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
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
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

