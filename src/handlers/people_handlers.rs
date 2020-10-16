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
use crate::handlers::{generate_cyto_graph, RenderPerson};

use crate::schema::{people, lenses, nodes};

#[get("/person/{id}")]
pub async fn person_page(
    web::Path(id): web::Path<i32>, 
    data: web::Data<AppData>, 
    _req:HttpRequest
) -> impl Responder {
    let mut ctx = Context::new(); 

    let p = People::find(id).unwrap();

    ctx.insert("user_code", &p.code);

    let title = format!("Person: {}", &p.code);
    ctx.insert("title", &title);
    
    // add pull for lens data
    let people_with_lenses = RenderPerson::from(p).expect("Unable to load lenses");

    ctx.insert("people_lenses", &people_with_lenses);

    let rendered = data.tmpl.render("person.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/person_network_graph/{id}")]
pub async fn person_graph(
    web::Path(id): web::Path<i32>,
    data: web::Data<AppData>
) -> impl Responder {
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let person: People = people::table.filter(people::id.eq(id))
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

    let mut node_vec = Vec::new();
    let mut lens_vec =  Vec::new();

    for (l, n) in node_lenses.into_iter() {
        lens_vec.push(l);
        node_vec.push(n);
    };

    node_vec.sort();
    lens_vec.sort();

    node_vec.dedup();
    lens_vec.dedup();
    
    let graph = generate_cyto_graph(people_vec, node_vec, lens_vec);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);

    let title = "Person Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

