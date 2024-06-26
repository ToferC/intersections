use std::{sync::Mutex};

use actix_web::{web, get, HttpResponse, Responder, HttpRequest, ResponseError};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};

use std::collections::HashMap;

use crate::models::{Experiences, Communities, People, User, Phrases};
use crate::models::{CytoGraph, generate_node_cyto_graph};


#[get("/{lang}/data_global_graph")]
pub async fn data_global_graph(
    path: web::Path<String>,
    // no longer using appdata to hold graph
    // this function is a placeholder in case we go back here
    data: web::Data<AppData>,
    graph: web::Data<Mutex<CytoGraph>>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let lang = path.into_inner();

    let g = graph.lock().unwrap().clone();
        
    let j = serde_json::to_string_pretty(&g).unwrap();

    drop(graph);
    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);
    
    let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/global_graph")]
pub async fn global_graph(
    path: web::Path<String>,
    data: web::Data<AppData>,
    
    id:Option<Identity>,
    req: HttpRequest,
  
) -> impl Responder {
    let lang = path.into_inner();
    let experience_vec = Experiences::find_all_real().expect("Unable to load experiences");

    // create vec of bridge connections from people
    let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();

    for l in &experience_vec {
        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.slug.to_owned()); 
    };

    // now instead of building AggExperience, we build the graph
    let graph = generate_node_cyto_graph(experience_vec, people_connections, None, &lang);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

    ctx.insert("graph_data", &j);

    let title = match lang.as_str() {
        "en" => "Global Network Graph",
        _ => "Graph du réseau mondial",
    };

    ctx.insert("title", &title);
    
    let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/full_community_graph/{community_slug}")]
pub async fn full_community_node_graph(
    data: web::Data<AppData>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
    id: Option<Identity>,
) -> impl Responder {
    let (lang, community_slug) = path.into_inner();

    // Validate community
    let community_result = Communities::find_from_slug(&community_slug);

    match community_result {
        Ok(community) => {
            
            let (mut ctx, session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path());

            let session_user_id = match User::find_id_from_slug(&session_user) {
                Ok(id) => id,
                Err(_e) => 9999,
            };

            // validate if user can view community graph
            if community.open || session_user_id == community.user_id {
                
                let experience_vec = Experiences::find_all().expect("Unable to load experiences");
    
                let community_people_ids = People::find_ids_from_community(community.id).expect("Unable to find community members");
            
                // create vec of bridge connections from people
                let mut people_connections: HashMap<i32, Vec<String>> = HashMap::new();
                let mut community_exp_vec: Vec<Experiences> = Vec::new();
            
                for l in &experience_vec {

                    if community_people_ids.contains(&l.person_id) {
                        // add to people connections and experience vec
                        people_connections.entry(l.person_id).or_insert(Vec::new()).push(l.slug.to_owned()); 
                        community_exp_vec.push(l.clone());
                    }
                };
            
                // now instead of building AggExperience, we build the graph
                let graph = generate_node_cyto_graph(community_exp_vec, people_connections, Some(community.slug), &lang);
            
                let j = serde_json::to_string_pretty(&graph).unwrap();
            
                ctx.insert("graph_data", &j);

                let community_name = Phrases::find(community.tag, &lang).expect("Unable to find phrase");
            
                let title = format!("Graph - {}", &community_name.text);
                
                ctx.insert("title", &title);
                
                let rendered = data.tmpl.render("graphs/node_network_graph.html", &ctx).unwrap();
                HttpResponse::Ok().body(rendered)
            } else {
                // user not validated to see community
                println!("User not validated to see community");
                return HttpResponse::Found().header("Location",format!("/{}", &lang)).finish()

            }
        },
        Err(e) => {
            println!("Error: {}", e);
            return e.error_response()
        },
    }      
}



