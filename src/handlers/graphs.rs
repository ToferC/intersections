use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use crate::AppData;
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};
use serde_json::json;

use num_bigint::{ToBigInt};
use bigdecimal::BigDecimal;

use crate::models::{NewPerson, Lens, Lenses, Node, Nodes, People};
use crate::database;
use crate::error_handler::CustomError;

use crate::schema::{people, lenses, nodes};

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let people_vec = People::find_all().expect("Unable to load people");
    
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
    
    let mut node_vec: Vec<Nodes> = vec![node];
        
    // join lenses and nodes
    let people_lenses: Vec<(Lenses, People)> = Lenses::belonging_to(&node_vec)
        .inner_join(people::table)
        .load::<(Lenses, People)>(&conn)
        .expect("Error leading people");
    
    // group node_lenses by people
    let grouped = people_lenses.grouped_by(&node_vec);
    
    // structure result
    let result: Vec<(Node, Vec<(Lenses, People)>)> = node_vec
        .into_iter()
        .zip(grouped)
        .collect();

    let j = serde_json::to_string(&result).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);
    
    let rendered = data.tmpl.render("full_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

