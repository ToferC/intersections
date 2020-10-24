use std::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::{ToBigInt};
use tera::Context;
use serde::{Deserialize, Serialize};

use crate::AppData;
use crate::models::{Lens, Lenses, NewPerson, People, Node, Nodes};
use crate::handlers::{CytoGraph, CytoNode, CytoEdge};
use crate::error_handler::CustomError;

#[derive(Deserialize, Debug)]
pub struct FirstLensForm {
    name: String,
    domain: String,
    response_1: String,
    response_2: String,
    response_3: String,
    inclusivity: BigDecimal,
    related_code: String,
}

#[derive(Deserialize, Debug)]
pub struct AddLensForm {
    name: String,
    domain: String,
    response_1: String,
    response_2: String,
    response_3: String,
    inclusivity: BigDecimal,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderPerson {
    pub person: People,
    pub lenses: Vec<Lenses>,
    pub total_inclusivity: f32,
}

impl RenderPerson {
    pub fn from(person: People) -> Result<Vec<Self>, CustomError> {

        let result = People::get_lenses(&person)?;

        let mut result_vec: Vec<RenderPerson> = Vec::new();

        for r in result {

            let mut total_inclusivity: BigDecimal = BigDecimal::new(0.to_bigint().unwrap(), 0);

            for l in &r.1 {
                total_inclusivity = total_inclusivity + &l.inclusivity;
            };

            let total_inclusivity = total_inclusivity.to_f32().unwrap();

            let total_inclusivity = (total_inclusivity * 100.0).round() / 100.0;
        
            let p = RenderPerson {
                person: r.0,
                lenses: r.1,
                total_inclusivity: total_inclusivity,
            };

            result_vec.push(p);
        }

        Ok(result_vec)
    }
}

#[get("/first_lens_form")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    let node_names = Nodes::find_all_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    

    let rendered = data.tmpl.render("first_lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/first_lens_form")]
pub async fn handle_lens_form_input(
    _data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
    req: HttpRequest, 
    form: web::Form<FirstLensForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", String::from("/first_lens_form")).finish()
    };

    let mut person = NewPerson::new();

    // Get related persons
    if &form.related_code != "" {
        person.related_codes.push(form.related_code.to_owned());
    };

    let node = Node::new(
        form.name.to_lowercase().to_owned(),
        form.domain.to_lowercase().to_owned(),
    );

    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_lowercase().to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_lowercase().to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_lowercase().to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);

    // Post person to db
    let new_person = People::create(&person.clone()).expect("Unable to add person to DB");

    // add person to graph representation
    let person_node = CytoGraph::add_person_node(&new_person);
    
    let mut g = graph.lock().expect("Unable to unlock graph");
    
    g.nodes.push(CytoNode {
            data: person_node,
        });

    drop(g);
    
    // Check if node exists, if not create it
    let nodes = Nodes::find_all().unwrap();

    let tn = nodes.iter().find(|n| n.node_name == node.node_name);
    
    let node_id: i32 = match &tn {
        // target exists
        Some(target) => {
            target.id
        }
        None => {
            // no target
            let new_node = Nodes::create(&node).expect("Unable to create node.");

            let node_rep = CytoGraph::add_node(&new_node);

            let mut g = graph.lock().expect("Unable to unlock graph");

            g.nodes.push(CytoNode {
                data: node_rep,
            });

            new_node.id
        }
    };
    
    // Insert lens to db
    let l = Lens::new(
        node.node_name.clone(),
        node.domain_token.clone(),
        new_person.id,
        node_id,
        lived_statements,
        inclusivity,
    );

    let new_lens = Lenses::create(&l).expect("Unable to create lens.");
    let lens_rep = CytoGraph::add_lens_edge(&new_lens);
    
    let mut g = graph.lock().expect("Unable to unlock graph");

    println!("Pre-lens length: {}", &g.edges.len());

    g.edges.push(CytoEdge {
        data: lens_rep,
    });

    println!("Post-lens length: {}", &g.edges.len());

    drop(g);

    HttpResponse::Found().header("Location", format!("/add_lens_form/{}", new_person.code)).finish()
}

#[get("/add_lens_form/{code}")]
pub async fn add_lens_form_handler(
    web::Path(code): web::Path<String>, 
    data: web::Data<AppData>, 
    _req:HttpRequest
) -> impl Responder {
    let mut ctx = Context::new(); 

    let p = People::find_from_code(&code).unwrap();

    ctx.insert("user_code", &p.code);
    ctx.insert("user_id", &p.id);

    let node_names = Nodes::find_all_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);

    // add pull for lens data
    let people_with_lenses = RenderPerson::from(p).expect("Unable to load lenses");

    ctx.insert("people_lenses", &people_with_lenses);

    let rendered = data.tmpl.render("add_lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/add_lens_form/{code}")]
pub async fn add_handle_lens_form_input(
    web::Path(code): web::Path<String>,
    _data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
    _req: HttpRequest, 
    form: web::Form<AddLensForm>
) -> impl Responder {

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", format!("/add_lens_form/{}", &code)).finish()
    };

    println!("Find person");
    let p = People::find_from_code(&code).unwrap();

    println!("Create Node");
    let node = Node::new(
        form.name.to_lowercase().to_owned(),
        form.domain.to_lowercase().to_owned(),
    );

    println!("Get statements");
    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_lowercase().to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_lowercase().to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_lowercase().to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);
    
    // Check if node exists, if not create it
    let nodes = Nodes::find_all().unwrap();

    let tn = nodes.iter().find(|n| n.node_name == node.node_name);
    
    let node_id: i32 = match &tn {
        Some(target) => {
            target.id
        }
        None => {
            // no target
            let new_node = Nodes::create(&node).expect("Unable to create node.");

            let node_rep = CytoGraph::add_node(&new_node);

            let mut g = graph.lock().expect("Unable to unlock graph");

            g.nodes.push(CytoNode {
                data: node_rep,
            });

            new_node.id
        }
    };
    
    let l = Lens::new(
        node.node_name.clone(),
        node.domain_token.clone(),
        p.id,
        node_id,
        lived_statements,
        inclusivity,
    );

    let new_lens = Lenses::create(&l).expect("Unable to create lens.");
    let lens_rep = CytoGraph::add_lens_edge(&new_lens);
    
    let mut g = graph.lock().expect("Unable to unlock graph");

    println!("Pre-lens length: {}", &g.edges.len());

    g.edges.push(CytoEdge {
        data: lens_rep,
    });

    println!("Post-lens length: {}", &g.edges.len());

    drop(g);

    HttpResponse::Found().header("Location", format!("/add_lens_form/{}", code)).finish()
}