

use actix_web::{web, get, HttpResponse, HttpRequest, Responder, ResponseError};
use actix_identity::Identity;
use inflector::Inflector;
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use std::collections::HashMap;

use crate::{AppData, generate_basic_context};
use crate::models::{Experiences, Nodes, Communities, User, People, Phrases};
use database;
use crate::models::{AggregateExperience, generate_node_cyto_graph};
use error_handler::error_handler::CustomError;

use crate::schema::{experiences};

#[get("/{lang}/node/{node_slug}")]
pub async fn node_page(
    path: web::Path<(String, String)>, 
    data: web::Data<AppData>, 
    
    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, node_slug) = path.into_inner();
    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    let mut conn = database::connection().expect("Unable to connect to db");
    
    let node_select = Nodes::find_by_slug(&node_slug);

    match node_select {
        Ok(node) => {

            // get connected nodes via people with experiencee connections to our prime node
            let experience_vec: Vec<Experiences> = Experiences::belonging_to(&node)
                .load::<Experiences>(&mut conn)
                .expect("Error leading connected experiences");
        
            let mut people_id_vec: Vec<i32> = Vec::new();
            let mut node_id_vec: Vec<i32> = Vec::new();
        
            for l in &experience_vec {
                people_id_vec.push(l.person_id);
                node_id_vec.push(l.node_id);
            };
        
            people_id_vec.sort();
            people_id_vec.dedup();
        
        
            // add experiences for the people connected by node
            let mut connected_experiences = experiences::table.filter(experiences::person_id.eq_any(&people_id_vec))
                .load::<Experiences>(&mut conn)
                .expect("Unable to load experiences");
            
            for l in &mut connected_experiences {
                node_id_vec.push(l.node_id);
            };
        
            node_id_vec.sort();
            node_id_vec.dedup();
            
            println!("nodes: {:?}, people: {:?}", &node_id_vec, &people_id_vec);
            
            let mut aggregate_experiences: Vec<AggregateExperience> = Vec::new();
        
            for i in node_id_vec {
                let mut temp_experience_vec: Vec<Experiences> = Vec::new();
        
                for l in &mut connected_experiences {
        
                    if i == l.node_id && i != node.id {
                        temp_experience_vec.push(l.clone());
                    }
                    // count people associated to multiple similar nodes
                    // show connections across the nodes and experiences
                };
        
                if temp_experience_vec.len() > 0 {
                    let agg_experiences = AggregateExperience::from(temp_experience_vec, &lang);
                    aggregate_experiences.push(agg_experiences);
                }
            };
        
            aggregate_experiences.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
            aggregate_experiences.dedup();
        
            // Aggregate info from experiences related to the prime node
            let node_experience = AggregateExperience::from(experience_vec, &lang);

            let phrases = Phrases::get_phrase_map(vec![node.node_name], &lang).expect("Unable to load node_title");
                
            ctx.insert("community_slug", "");
        
            ctx.insert("node", &node);
            ctx.insert("phrases", &phrases);
            
            ctx.insert("node_experience", &node_experience);
        
            ctx.insert("other_experiences", &aggregate_experiences);
        
            let rendered = data.tmpl.render("nodes/node.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("Error: {}", err);
            return err.error_response();
        },
    };
    
}

#[get("/{lang}/community_node/{community_slug}/{node_slug}")]
pub async fn community_node_page(
    path: web::Path<(String, String, String)>, 
    data: web::Data<AppData>, 
    
    req:HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, community_slug, node_slug) = path.into_inner();

    let (mut ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    // validate user has rights to view
    let community_result = Communities::find_from_slug(&community_slug);

    match community_result {
        Ok(community) => {

            let mut owner = false;
        
            if &session_user != "" {
                let user = User::find_from_slug(&session_user);
        
                if community.user_id == user.unwrap().id {
                    owner = true;
                }
            
                // Redirect if community is closed and user isn't community owner
                if !community.open && !owner {
                    return HttpResponse::Found().header("Location", String::from("/")).finish()
                };
            } else {
                if !community.open {
                    return HttpResponse::Found().header("Location", String::from("/")).finish()
                }
            };
        
            ctx.insert("owner", &owner);
            
            // connect to db --> move this into helper function
            let mut conn = database::connection().expect("Unable to connect to db");
            
            // get ids of people in the community
            let community_people_ids = People::find_ids_from_community(community.id).expect("Unable to find community members");
            
            let node_select = Nodes::find_by_slug(&node_slug);

            match node_select {
                Ok(node) => {

                    // get connected nodes via people with experience connections to our prime node and community
                    let experience_vec: Vec<Experiences> = experiences::table
                        .filter(experiences::person_id.eq_any(&community_people_ids)
                            .and(experiences::node_id.eq(&node.id)))
                        .load::<Experiences>(&mut conn)
                        .expect("Error loading connected experiences");
                
                    let mut people_id_vec: Vec<i32> = Vec::new();
                    let mut node_id_vec: Vec<i32> = Vec::new();
                
                    
                    // add ids to people_id_vec and node_id_vec if they are in the community
                    for l in &experience_vec {
                            people_id_vec.push(l.person_id);
                            node_id_vec.push(l.node_id);
                    };
                
                    people_id_vec.sort();
                    people_id_vec.dedup();
                
                    println!("People ID VEC: {:?}", &people_id_vec);
                
                    // add experiences for the people connected by node
                    let mut connected_experiences = experiences::table
                        .filter(experiences::person_id.eq_any(&people_id_vec))
                        .load::<Experiences>(&mut conn)
                        .expect("Unable to load experiences");
                    
                    for l in &mut connected_experiences {
                        node_id_vec.push(l.node_id);
                    };
                
                    node_id_vec.sort();
                    node_id_vec.dedup();
                    
                    println!("nodes: {:?}, people: {:?}", &node_id_vec, &people_id_vec);
                    
                    let mut aggregate_experiences: Vec<AggregateExperience> = Vec::new();
                
                    for i in node_id_vec {
                        let mut temp_experience_vec: Vec<Experiences> = Vec::new();
                
                        for l in &mut connected_experiences {
                
                            if i == l.node_id && i != node.id {
                                temp_experience_vec.push( Experiences {
                                    id: l.id,
                                    node_name: l.node_name.to_owned(),
                                    node_domain: l.node_domain.to_owned(),
                                    importance: l.importance,
                                    person_id: l.person_id,
                                    node_id: l.node_id,
                                    date_created: l.date_created,
                                    statements: l.statements.clone(),
                                    inclusivity: l.inclusivity.clone(),
                                    slug: l.slug.clone(),
                                });
                            }
                            // count people associated to multiple similar nodes
                            // show connections across the nodes and experiences
                        };
                
                        if temp_experience_vec.len() > 0 {
                            let agg_experiences = AggregateExperience::from(temp_experience_vec, &lang);
                            aggregate_experiences.push(agg_experiences);
                        }
                    };
                
                    aggregate_experiences.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
                    aggregate_experiences.dedup();
                
                    // Aggregate info from experiences related to the prime node
                    let node_experience = AggregateExperience::from(experience_vec, &lang);
                
                    ctx.insert("title", &format!("{}", &node_experience.name));
                
                    ctx.insert("node", &node);

                    let phrases = Phrases::get_phrase_map(vec![node.node_name, community.tag], &lang).expect("Unable to load node_title");
                
                    ctx.insert("phrases", &phrases);
                    
                    ctx.insert("node_experience", &node_experience);
                
                    ctx.insert("other_experiences", &aggregate_experiences);
                
                    ctx.insert("community", &community);
                        
                    let rendered = data.tmpl.render("nodes/community_node.html", &ctx).unwrap();
                    HttpResponse::Ok().body(rendered)
                },
                Err(err) => {
                    println!("{}", err);
                    return err.error_response()
                },
            }
        },
        Err(err) => {
            println!("Error: {}", err);
            return err.error_response();
        },
    }
}

#[get("/{lang}/node_graph/{node_slug}")]
pub async fn node_graph(
    path: web::Path<(String, String)>,
    data: web::Data<AppData>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, node_slug) = path.into_inner();

    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    let mut conn = database::connection().expect("Unable to connect to db");
    
    let node_select = Nodes::find_by_slug(&node_slug);

    match node_select {
        Ok(node) => {

            // get connected nodes via people with experiencee connections to our prime node
            let mut experience_vec: Vec<Experiences> = Experiences::belonging_to(&node)
                .load::<Experiences>(&mut conn)
                .expect("Error leading connected experiences");
        
            let mut people_id_vec: Vec<i32> = Vec::new();
            let mut node_id_vec: Vec<i32> = Vec::new();
        
            // create vec of bridge connections from people
            let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();
        
            for l in &experience_vec {
                people_id_vec.push(l.person_id);
                node_id_vec.push(l.node_id);
            };
        
        
            people_id_vec.sort();
            people_id_vec.dedup();
        
            // add experiences for the people connected by node
            let mut connected_experiences = experiences::table.filter(experiences::person_id.eq_any(&people_id_vec))
                .load::<Experiences>(&mut conn)
                .expect("Unable to load experiences");
            
            experience_vec.append(&mut connected_experiences);
        
            for l in &experience_vec {
                people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.slug.to_owned()); 
            };
        
            println!("{:?}", &people_connections);
        
            for l in &mut connected_experiences {
                node_id_vec.push(l.node_id);
            };
        
            // now instead of building AggregateExperience, we build the graph
            let graph = generate_node_cyto_graph(experience_vec, people_connections, None, &lang);
        
            let j = serde_json::to_string_pretty(&graph).unwrap();
            
            ctx.insert("graph_data", &j);

            let node_name = Phrases::find(node.node_name, &lang).expect("Unable to load node_title");
            ctx.insert("node_name", &node_name.text);
        
            let title = "Node Network Graph";
            ctx.insert("title", title);
            
            let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(err) => {
            println!("Error {}", &err);
            err.error_response()
        },
    }
    
}

#[get("/{lang}/community_node_graph/{community_slug}/{node_slug}")]
pub async fn community_node_graph(
    // Rework this as a connected node graph
    path: web::Path<(String, String, String)>, 
    data: web::Data<AppData>,
    
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, community_slug, node_slug) = path.into_inner();

    
    let (mut ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    let community_select = Communities::find_from_slug(&community_slug);

    match community_select {
        Ok(community) => {

            let mut owner = false;
        
            if &session_user != "" {
                let user = User::find_from_slug(&session_user);
        
                if community.user_id == user.unwrap().id {
                    owner = true;
                }
            
                // Redirect if community is closed and user isn't community owner
                if !community.open && !owner {
                    let err = CustomError::new(
                        406,
                        "Not authorized".to_string(),
                    );
                    println!("{}", &err);
                    return err.error_response()
                };
            } else {
                if !community.open {
                    let err = CustomError::new(
                        406,
                        "Not authorized".to_string(),
                    );
                    println!("{}", &err);
                    return err.error_response()
                }
            };
        
            ctx.insert("owner", &owner);
        
            let mut conn = database::connection().expect("Unable to connect to db");
            
            let node_select = Nodes::find_by_slug(&node_slug);

            match node_select {
                Ok(node) => {

                    // get connected nodes via people with experiencee connections to our prime node
                    let mut experience_vec: Vec<Experiences> = Experiences::belonging_to(&node)
                        .load::<Experiences>(&mut conn)
                        .expect("Error leading connected experiences");
                    
                    let mut people_id_vec: Vec<i32> = Vec::new();
                    let mut node_id_vec: Vec<i32> = Vec::new();
                    
                    
                    for l in &experience_vec {
                        people_id_vec.push(l.person_id);
                        node_id_vec.push(l.node_id);
                    };
                    
                    people_id_vec.sort();
                    people_id_vec.dedup();
                    
                    // add experiences for the people connected by node
                    let mut connected_experiences = experiences::table.filter(experiences::person_id.eq_any(&people_id_vec))
                        .load::<Experiences>(&mut conn)
                        .expect("Unable to load experiences");
                    
                    experience_vec.append(&mut connected_experiences);
                    
                    // create vec of bridge connections from people
                    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();
                    
                    // get ids of people in the community
                    let community_people_ids = People::find_ids_from_community(community.id).expect("Unable to find community members");
                    
                    // add people connections from the community only
                    for l in &experience_vec {
                        if community_people_ids.contains(&l.person_id) {
                            people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.slug.to_owned()); 
                        }
                    };
                    
                    println!("{:?}", &people_connections);
                
                    for l in &mut connected_experiences {
                        node_id_vec.push(l.node_id);
                    };
                
                    // now instead of building AggregateExperience, we build the graph
                    let graph = generate_node_cyto_graph(experience_vec, people_connections, Some(community.slug.clone()), &lang);
                
                    let j = serde_json::to_string_pretty(&graph).unwrap();
                    
                    ctx.insert("graph_data", &j);

                    let node_name = Phrases::find(node.node_name, &lang).expect("Unable to load node_title");
                    ctx.insert("node_name", &node_name.to_owned());
                
                    ctx.insert("community", &community);
                    let community_name = Phrases::find(community.tag, &lang).expect("Unable to find phrase");
                    ctx.insert("community_name", &community_name.text);
                
                    let title = format!("Graph - {} - {}", &node_name.text.to_title_case(), &community_name.text);
                    ctx.insert("title", &title);

                
                    let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
                    HttpResponse::Ok().body(rendered)
                },
                Err(err) => {
                    println!("{}", &err);
                    return err.error_response()
                },
            }
        },
        Err(err) => {
            println!("{}", err);
            return err.error_response()
        }
    }

}