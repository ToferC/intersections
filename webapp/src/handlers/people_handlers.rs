use std::sync::Mutex;

use actix_web::{web, get, post, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, extract_identity_data};
use tera::{Context};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use crate::models::{Experiences, Nodes, People, Communities, CommunityData, generate_cyto_graph};
use database;
use crate::handlers::{RenderPerson, DeleteForm};
use crate::models::AggregateExperience;

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

    ctx.insert("person", &p);

    let community = Communities::find(p.community_id).expect("Unable to find community");
    ctx.insert("community", &community);

    let title = "Profile Page";
    ctx.insert("title", &title);
    
    // add pull for experience data
    let people_with_experiences = RenderPerson::from(&p, true).expect("Unable to load experiences");

    ctx.insert("people_experiences", &people_with_experiences);

    let mut aggregate_experiences: Vec<AggregateExperience> = Vec::new();

    for p in people_with_experiences.into_iter() {
        for l in p.experiences {
            let node = Nodes::find(l.node_id).expect("Unable to load experiences");
            let experiences = Experiences::find_from_node_id(node.id).expect("Unable to load experiences");
            let agg_experiences = AggregateExperience::from(experiences);
            aggregate_experiences.push(agg_experiences);
        }
    };

    aggregate_experiences.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
    aggregate_experiences.dedup();

    ctx.insert("other_experiences", &aggregate_experiences);

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
    
    // join experiences and nodes
    let node_experiences: Vec<(Experiences, Nodes)> = Experiences::belonging_to(&people_vec)
        .inner_join(nodes::table)
        .load::<(Experiences, Nodes)>(&conn)
        .expect("Error leading people");

    let mut node_vec = Vec::new();
    let mut experience_vec =  Vec::new();

    for (l, n) in node_experiences.into_iter() {
        experience_vec.push(l);
        node_vec.push(n);
    };

    node_vec.sort();
    experience_vec.sort();

    node_vec.dedup();
    experience_vec.dedup();
    
    let graph = generate_cyto_graph(people_vec, node_vec, experience_vec, None);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    ctx.insert("graph_data", &j);

    let title = "Person Network Graph";
    ctx.insert("title", title);

    // add node_names for navbar drop down
    ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());
    
    let rendered = data.tmpl.render("graphs/network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/delete_person/{code}")]
pub async fn delete_person(
    web::Path(code): web::Path<String>,
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (session_user, role) = extract_identity_data(&id);

    let person = People::find_from_code(&code);
        
    match person {
        Ok(person) => {
            let mut ctx = Context::new();

            ctx.insert("person", &person);
        
            ctx.insert("session_user", &session_user);
            ctx.insert("role", &role);

            // add node_names for navbar drop down
            ctx.insert("node_names", &node_names.lock().expect("Unable to unlock").clone());

            let rendered = data.tmpl.render("people/delete_person.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(c) => {
            // no user returned for ID
            println!("{}", c);
            return HttpResponse::Found().header("Location", "/").finish()
        },
    }
}

#[post("/delete_person/{code}")]
pub async fn delete_person_post(
    web::Path(code): web::Path<String>,
    _data: web::Data<AppData>,
    _req: HttpRequest,
    _id: Identity,
    form: web::Form<DeleteForm>,
) -> impl Responder {

    let person = People::find_from_code(&code);
        
    match person {
        Ok(person) => {
            if form.verify.trim().to_string() == "Delete my profile" {
                println!("matches verify string - deleting person");

                // remove data from community
                let mut community = Communities::find(person.community_id).expect("Unable to load community");

                // get the data specific to this profile
                let person_with_experiences = RenderPerson::from(&person, false).expect("Unable to load experiences");

                let comm_data: CommunityData = serde_json::from_value(community.data).unwrap();

                let mut comm_data = comm_data.to_owned();

                comm_data.members -= 1;
                
                for experience in &person_with_experiences[0].experiences {
                    comm_data.experiences -= 1;
                    comm_data.inclusivity_map.remove(&experience.id);
                };

                let total: f32 = comm_data.inclusivity_map.values().sum();

                comm_data.mean_inclusivity = total / comm_data.inclusivity_map.len() as f32;

                community.data = serde_json::to_value(comm_data).unwrap();

                let update = Communities::update(&community);

                match update {
                    Ok(c) => {
                        println!("Community {} updated", c.tag);
                    },
                    Err(e) => {
                        println!("Community update failed: {}", e);
                    }
                };

                // delete person
                People::delete(person.id).expect("Unable to delete person");
                return HttpResponse::Found().header("Location", "/").finish()
            } else {
                println!("User does not match verify string - return to delete page");
                return HttpResponse::Found().header("Location", format!("/delete_person/{}", &person.code)).finish()
            };
        },
        Err(err) => {
            // no user returned for ID
            println!("{}", err);
            return HttpResponse::Found().header("Location", "/").finish()
        },
    };
}

