use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get};
use bigdecimal::BigDecimal;
use num_bigint::{ToBigInt};
use tera::Context;
use serde::Deserialize;

use crate::AppData;
use crate::models::{Lens, Lenses, NewPerson, People, Node, Nodes};

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

#[get("/first_lens_form")]
pub async fn lens_form_handler(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("first_lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/first_lens_form")]
pub async fn handle_lens_form_input(
    _data: web::Data<AppData>, 
    req: HttpRequest, 
    form: web::Form<FirstLensForm>
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    println!("{:?}", form);

    let mut person = NewPerson::new();

    // Get related persons
    if &form.related_code != "" {
        person.related_codes.push(form.related_code.to_owned());
    };

    let node = Node::new(
        form.name.to_owned(),
        form.domain.to_owned(),
    );

    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);

    // Post person to db
    let new_person = People::create(&person.clone()).expect("Unable to add person to DB");
    
    // Check if node exists, if not create it
    let nodes = Nodes::find_all().unwrap();

    let tn = nodes.iter().find(|n| n.node_name == node.node_name);
    
    let node_id: i32 = match &tn {
        Some(target) => {
            target.id
        }
        None => {
            let new_node = Nodes::create(&node).expect("Unable to create node.");
            new_node.id
        }
    };
    
    // Insert lens to db
    let l = Lens::new(
        new_person.id,
        node_id,
        lived_statements,
        inclusivity,
    );

    let new_lens = Lenses::create(&l).expect("Unable to create lens.");
    
    println!("{:?} -- {:?} -- {:?}", new_lens, &new_person, node);

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

    let rendered = data.tmpl.render("add_lens_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/add_lens_form/{code}")]
pub async fn add_handle_lens_form_input(
    web::Path(code): web::Path<String>,
    _data: web::Data<AppData>, 
    _req: HttpRequest, 
    form: web::Form<AddLensForm>
) -> impl Responder {

    println!("Find person");
    let p = People::find_from_code(&code).unwrap();

    println!("Create Node");
    let node = Node::new(
        form.name.to_owned(),
        form.domain.to_owned(),
    );

    println!("Get statements");
    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_owned());
    };

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);
    
    // Check if node exists, if not create it
    println!("Get Nodes");
    let nodes = Nodes::find_all().unwrap();

    let tn = nodes.iter().find(|n| n.node_name == node.node_name);
    
    let node_id: i32 = match &tn {
        Some(target) => {
            target.id
        }
        None => {
            let new_node = Nodes::create(&node).expect("Unable to create node.");
            new_node.id
        }
    };
    
    println!("Add lens");
    // Insert lens to db
    let l = Lens::new(
        p.id,
        node_id,
        lived_statements,
        inclusivity,
    );

    let new_lens = Lenses::create(&l).expect("Unable to create lens.");
    
    println!("{:?} -- {:?} -- {:?}", new_lens, &p, node);

    println!("Forward");
    HttpResponse::Found().header("Location", format!("/add_lens_form/{}", code)).finish()
}