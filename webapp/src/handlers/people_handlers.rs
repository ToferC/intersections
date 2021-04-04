use std::sync::Mutex;

use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, extract_identity_data};
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Lenses, Nodes, People, Communities, generate_cyto_graph};
use database;
use crate::handlers::{RenderPerson};
use crate::models::AggregateLens;

use crate::schema::{people, nodes};

#[get("/person/{code}")]
pub async fn person_page(
    web::Path(code): web::Path<String>, 
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let p = People::find_from_code(&code).unwrap();

    ctx.insert("person_code", &code);

    ctx.insert("person_id", &p.id);

    let community = Communities::find(p.community_id).expect("Unable to find community");
    ctx.insert("community", &community);

    let title = "Profile Page";
    ctx.insert("title", &title);
    
    // add pull for lens data
    let people_with_lenses = RenderPerson::from(p).expect("Unable to load lenses");

    ctx.insert("people_lenses", &people_with_lenses);

    let mut aggregate_lenses: Vec<AggregateLens> = Vec::new();

    for p in people_with_lenses.into_iter() {
        for l in p.lenses {
            let node = Nodes::find(l.node_id).expect("Unable to load lenses");
            let lenses = Lenses::find_from_node_id(node.id).expect("Unable to load lenses");
            let agg_lenses = AggregateLens::from(lenses);
            aggregate_lenses.push(agg_lenses);
        }
    };

    aggregate_lenses.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
    aggregate_lenses.dedup();

    ctx.insert("other_lenses", &aggregate_lenses);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

    let rendered = data.tmpl.render("people/person.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/person_network_graph/{person_id}")]
pub async fn person_graph(
    web::Path(person_id): web::Path<i32>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let mut ctx = Context::new();

    // Get session data and add to context
    let (session_user, role) = extract_identity_data(&id);
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);
    
    let conn = database::connection().expect("Unable to connect to db");
    
    let person: People = people::table.filter(people::id.eq(person_id))
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
    
    let graph = generate_cyto_graph(people_vec, node_vec, lens_vec, None);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    ctx.insert("graph_data", &j);

    let title = "Person Network Graph";
    ctx.insert("title", title);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
    let rendered = data.tmpl.render("graphs/network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

