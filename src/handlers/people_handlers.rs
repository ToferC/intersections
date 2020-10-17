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
use crate::handlers::{generate_cyto_graph, RenderPerson};

use crate::schema::{people, nodes};

#[derive(Serialize, Debug)]
pub struct AggregateLenses {
    pub name: String,
    pub domain: String,
    pub count: u32,
    pub mean_inclusivity: f32,
    pub frequency_distribution: BTreeMap<String, u32>,
}

impl AggregateLenses {
    pub fn from(lenses: Vec<Lenses>) -> AggregateLenses {
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

        AggregateLenses {
            name: name.to_owned(),
            domain: domain.to_owned(),
            count: count,
            mean_inclusivity: inclusivity / count as f32,
            frequency_distribution: counts,
        }
    }
}

#[get("/person/{id}")]
pub async fn person_page(
    web::Path(id): web::Path<i32>, 
    data: web::Data<AppData>, 
    _req:HttpRequest
) -> impl Responder {
    let mut ctx = Context::new(); 

    let p = People::find(id).unwrap();

    ctx.insert("person_id", &p.id);

    let title = format!("Person: P-{}", &p.id);
    ctx.insert("title", &title);
    
    // add pull for lens data
    let people_with_lenses = RenderPerson::from(p).expect("Unable to load lenses");

    ctx.insert("people_lenses", &people_with_lenses);

    let mut aggregate_lenses: Vec<AggregateLenses> = Vec::new();

    for p in people_with_lenses.into_iter() {
        for l in p.lenses {
            let node = Nodes::find(l.node_id).expect("Unable to load lenses");
            let lenses = Lenses::find_from_node_id(node.id).expect("Unable to load lenses");
            let agg_lenses = AggregateLenses::from(lenses);
            aggregate_lenses.push(agg_lenses);
        }
    };

    ctx.insert("other_lenses", &aggregate_lenses);

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

