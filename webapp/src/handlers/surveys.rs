use std::{sync::{Mutex, MutexGuard, Arc}};
use actix_web::{web, HttpRequest, HttpResponse, Responder, post, get, ResponseError};
use bigdecimal::{BigDecimal, ToPrimitive};
use actix_identity::Identity;
use tokio::{spawn};
use futures::future::{lazy};
use inflector::Inflector;
use num_bigint::{ToBigInt};
use serde::{Deserialize, Serialize};

use crate::{AppData, generate_basic_context};
use crate::models::{Experience, Experiences, RawExperience,
    NewPerson, People, Node, Nodes, Communities, CommunityData, 
    Phrases, translate_experience_phrases};
use error_handler::error_handler::CustomError;

#[derive(Deserialize, Debug)]
pub struct FirstExperienceForm {
    name: String,
    domain: String,
    response_1: String,
    response_2: String,
    response_3: String,
    inclusivity: BigDecimal,
    related_code: String,
}

#[derive(Deserialize, Debug)]
pub struct AddExperienceForm {
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
    pub experiences: Vec<(Experiences, Vec<Phrases>)>,
    pub total_inclusivity: f32,
}

impl RenderPerson {
    pub fn from(person: &People, related: bool, lang: &str) -> Result<Vec<Self>, CustomError> {

        let result = People::get_experiences(&person, related, &lang)?;

        let mut result_vec: Vec<RenderPerson> = Vec::new();

        for r in result {

            let mut total_inclusivity: BigDecimal = BigDecimal::new(0.to_bigint().unwrap(), 0);

            for (l, _p) in &r.1 {
                total_inclusivity = total_inclusivity + &l.inclusivity;
            };

            let total_inclusivity = total_inclusivity.to_f32().unwrap();

            let total_inclusivity = (total_inclusivity * 100.0).round() / 100.0;
        
            let p = RenderPerson {
                person: r.0,
                experiences: r.1.clone(),
                total_inclusivity: total_inclusivity,
            };

            result_vec.push(p);
        }

        Ok(result_vec)
    }
}

#[get("/{lang}/survey_intro/{community_code}")]
pub async fn survey_intro(
    data: web::Data<AppData>,
    web::Path((lang, community_code)): web::Path<(String, String)>, 
    req:HttpRequest,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    id: Identity,
) -> impl Responder {
    println!("Access index");

    // Validate community
    let community_result = Communities::find_from_code(&community_code);
    
    match community_result {
        Ok(community) => {
            let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);
        
            // all names for form autocomplete
            let all_node_names = Nodes::find_all_names(&lang).expect("Unable to load node names");
            ctx.insert("all_node_names", &all_node_names);

            ctx.insert("community", &community);
            
            let rendered = data.tmpl.render("survey/survey_intro.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error: {}", e);
            return e.error_response()
        }
    }

}

#[get("/{lang}/first_experience_form/{community_code}")]
pub async fn experience_form_handler(
    data: web::Data<AppData>,
    web::Path((lang, community_code)): web::Path<(String, String)>, 
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    
    // Validate community
    let community_result = Communities::find_from_code(&community_code);
    
    match community_result {
        Ok(community) => {
            let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);
        
            // all names for form autocomplete
            let all_node_names = Nodes::find_all_names(&lang).expect("Unable to load node names");
            ctx.insert("all_node_names", &all_node_names);

            ctx.insert("community", &community);
            
            let rendered = data.tmpl.render("survey/first_experience_form.html", &ctx).unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error: {}", e);
            return e.error_response()
        }
    }

}

#[post("/{lang}/first_experience_form/{community_code}")]
pub async fn handle_experience_form_input(
    _data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    web::Path((lang, community_code)): web::Path<(String, String)>, 
    req: HttpRequest, 
    form: web::Form<FirstExperienceForm>,
) -> impl Responder {
    println!("Handling Post Request: {:?}", req);

    let community_result = Communities::find_from_code(&community_code);

    match community_result {
        Ok(mut community) => {
            
                // validate form has data or re-load form
                if form.name.is_empty() || form.response_1.is_empty() {
                    return HttpResponse::Found().header("Location", format!("/{}/first_experience_form/{}", &lang, &community_code)).finish()
                };
            
                // associate person to community
                let mut person = NewPerson::new(community.id);
            
                // Get related persons
                if &form.related_code != "" {
                    person.related_codes.push(form.related_code.trim().to_owned());
                };

                let node_name = form.name.to_lowercase().trim().to_owned();

                let mut raw_exp = RawExperience::new(
                    node_name.to_owned(),
                    Vec::new(),
                );
            
                let mut lived_statements = vec!();
            
                if &form.response_1 != "" {
                    lived_statements.push(form.response_1.to_lowercase().trim().to_owned());
                    raw_exp.statements.push(form.response_1.to_lowercase().trim().to_owned());
                };
            
                if &form.response_2 != "" {
                    lived_statements.push(form.response_2.to_lowercase().trim().to_owned());
                    raw_exp.statements.push(form.response_2.to_lowercase().trim().to_owned());
                };
            
                if &form.response_3 != "" {
                    lived_statements.push(form.response_3.to_lowercase().trim().to_owned());
                    raw_exp.statements.push(form.response_3.to_lowercase().trim().to_owned());
                };

                // generate phrases for experience and node
                let _phrase_ids = raw_exp.generate_experience_phrases(&lang)
                    .await
                    .expect("Unable to generate phrases for experience");

                // translate user strings and map to existing phrases
                // async translation thread
                let copy_raw_exp = raw_exp.clone();
                let c = Arc::new(copy_raw_exp);
                
                let l = Arc::new(lang.clone());

                println!("Sending experience to translation");
                let _translations = spawn(
                    translate_experience_phrases(c, l));

                let node = Node::new(
                    raw_exp.name_id,
                    form.name.to_lower_case().trim().to_owned(),
                    form.domain.to_lowercase().trim().to_owned(),
                );
                
                let inclusivity = &form.inclusivity;
            
                let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);
            
                // Post person to db
                let new_person = People::create(&person.clone()).expect("Unable to add person to DB");
                
                // Check if node exists, if not create it
                let result = Nodes::find_by_slug(&node.slug, &lang);
                            
                let node= match result {
                    // target exists
                    Ok((target, _p)) => {
                        target
                    }
                    Err(e) => {
                        // no target
                        println!("{}", e);
                        let new_node = Nodes::create(&node).expect("Unable to create node.");
            
                        let mut temp_data: MutexGuard<Vec<(String, String)>> = node_names.lock().expect("Unable to unlock node_names");
            
                        temp_data.push((node_name.clone(), new_node.slug.clone()));
            
                        drop(temp_data);
            
                        new_node
                    }
                };
                
                // Insert experience to db
                let l = Experience::new(
                    node.node_name,
                    node.domain_token.clone(),
                    new_person.id,
                    node.id,
                    raw_exp.phrase_ids,
                    inclusivity.to_owned(),
                    node.slug.to_owned(),
                );
            
                let new_experience = Experiences::create(&l).expect("Unable to create experience.");
                
                /*
                let experience_rep = GEdge::from_experience(&new_experience);
                
                let mut g = graph.lock().expect("Unable to unlock graph");
            
                println!("Pre-experience length: {}", &g.edges.len());
            
                g.edges.push(CytoEdge {
                    data: experience_rep,
                });
            
                println!("Post-experience length: {}", &g.edges.len());
            
                drop(g);
                */

                // update the community based on new data
                let comm_data: CommunityData = serde_json::from_value(community.data).unwrap();

                let mut comm_data = comm_data.to_owned();

                comm_data.members += 1;
                comm_data.experiences += 1;
                comm_data.inclusivity_map.insert(new_experience.id, inclusivity.to_f32().unwrap());

                let total: f32 = comm_data.inclusivity_map.values().sum();

                comm_data.mean_inclusivity = total / comm_data.experiences as f32;

                community.data = serde_json::to_value(comm_data).unwrap();

                let update = Communities::update(&community);

                match update {
                    Ok(c) => {
                        println!("Community {} updated", c.tag);
                        return HttpResponse::Found().header("Location", format!("/{}/add_experience_form/{}", &lang, new_person.code)).finish()
                    },
                    Err(e) => {
                        println!("Community update failed: {}", e);
                        return e.error_response()
                    }
                };
        },
        Err(e) => {
            // invalid code - redirect
            println!("Error: {}", e);
            return e.error_response()
        }
    }
}

#[get("/{lang}/add_experience_form/{code}")]
pub async fn add_experience_form_handler(
    web::Path((lang, code)): web::Path<(String, String)>, 
    data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    req:HttpRequest,
    id: Identity,
) -> impl Responder {
    let (mut ctx, _session_user, _role, _lang) = generate_basic_context(id, &lang, req.uri().path(), node_names);

    let p = People::find_from_code(&code).unwrap();

    ctx.insert("user_code", &p.code);
    ctx.insert("user_id", &p.id);

    // all names for form autocomplete
    let all_node_names = Nodes::find_all_names(&lang).expect("Unable to load names");
    ctx.insert("all_node_names", &all_node_names);

    // add pull for experience data
    let people_with_experiences = RenderPerson::from(&p, true, &lang).expect("Unable to load experiences");
    ctx.insert("people_experiences", &people_with_experiences);

    // translate to fluent keys
    let helper_options = vec![
        "two", 
        "three", 
        "four", 
        "You may wish to add an experience related to your ability or a permanent or temporary disability.", 
        "You are creating a true intersectional profile. You may wish to add an experience related to your current state of mental health or age.",
        "You may wish to add an experience related to your body perception, personality type or caregiving responsibilities.",
        "You may wish to add an experience related to a life experience or stress that you going through.",
        "Amazing. You’ve got the hang of this. Add as many additional experiences as you like.",
    ];

    let helper_text = helper_options[(people_with_experiences.last().unwrap().experiences.len() -1).min(7)];
    ctx.insert("helper_text", &helper_text);

    let rendered = data.tmpl.render("survey/add_experience_form.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/add_experience_form/{code}")]
pub async fn add_handle_experience_form_input(
    web::Path((lang, code)): web::Path<(String, String)>,
    _data: web::Data<AppData>,
    node_names: web::Data<Mutex<Vec<(String, String)>>>,
    _req: HttpRequest, 
    form: web::Form<AddExperienceForm>,
) -> impl Responder {

    // validate form has data or re-load form
    if form.name.is_empty() || form.response_1.is_empty() {
        return HttpResponse::Found().header("Location", format!("/{}/add_experience_form/{}", &lang, &code)).finish()
    };

    println!("Find person");
    let p = People::find_from_code(&code).unwrap();

    let node_name = form.name.to_lowercase().trim().to_owned();

    let mut raw_exp = RawExperience::new(node_name.to_owned(), Vec::new());

    let mut lived_statements = vec!();

    if &form.response_1 != "" {
        lived_statements.push(form.response_1.to_lowercase().trim().to_owned());
        raw_exp.statements.push(form.response_1.to_lowercase().trim().to_owned());
    };

    if &form.response_2 != "" {
        lived_statements.push(form.response_2.to_lowercase().trim().to_owned());
        raw_exp.statements.push(form.response_2.to_lowercase().trim().to_owned());
    };

    if &form.response_3 != "" {
        lived_statements.push(form.response_3.to_lowercase().trim().to_owned());
        raw_exp.statements.push(form.response_3.to_lowercase().trim().to_owned());
    };

    let _phrase_ids = raw_exp.generate_experience_phrases(&lang)
        .await
        .expect("Unable to generate phrases for experience");

    // async translation thread
    let copy_raw_exp = raw_exp.clone();
    let c = Arc::new(copy_raw_exp);
    
    let l = Arc::new(lang.clone());

    println!("Sending experience to translation");
    let _translations = spawn(
        translate_experience_phrases(c, l));

    let node = Node::new(
        raw_exp.name_id,
        form.name.to_lower_case().trim().to_owned(),
        form.domain.to_lowercase().trim().to_owned(),
    );

    let inclusivity = &form.inclusivity;

    let inclusivity = BigDecimal::new(inclusivity.to_bigint().unwrap(), 2);
    
    // Check if node exists, if not create it
    let result = Nodes::find_by_slug(&node.slug, &lang);
    
    let node = match result {
        Ok((target, _p)) => {
            target
        }
        Err(e) => {
            println!("{}", e);

            // no target
            let new_node = Nodes::create(&node).expect("Unable to create node.");

            /*
            let node_rep = GNode::from_node(&new_node, &None);

            let mut g = graph.lock().expect("Unable to unlock graph");

            g.nodes.push(CytoNode {
                data: node_rep,
            });

            */

            // add node_names to appData
            let mut nn = node_names.lock().expect("Unable to unlock");
            nn.push((node_name.clone(), new_node.slug.clone()));
            drop(nn);

            new_node
        }
    };
    
    let l = Experience::new(
        node.node_name,
        node.domain_token.clone(),
        p.id,
        node.id,
        raw_exp.phrase_ids,
        inclusivity.to_owned(),
        node.slug.to_owned(),
    );

    let new_experience = Experiences::create(&l).expect("Unable to create experience.");
    
    /*
    let experience_rep = GEdge::from_experience(&new_experience);
    
    let mut g = graph.lock().expect("Unable to unlock graph");

    println!("Pre-experience length: {}", &g.edges.len());

    g.edges.push(CytoEdge {
        data: experience_rep,
    });

    println!("Post-experience length: {}", &g.edges.len());

    drop(g);
    */

    // update the community based on new data
    let mut community = Communities::find(p.community_id).expect("Unable to load community");

    let comm_data: CommunityData = serde_json::from_value(community.data).unwrap();

    let mut comm_data = comm_data.to_owned();

    comm_data.experiences += 1;
    comm_data.inclusivity_map.insert(new_experience.id, inclusivity.to_f32().unwrap());

    let total: f32 = comm_data.inclusivity_map.values().sum();

    comm_data.mean_inclusivity = total / comm_data.inclusivity_map.len() as f32;

    community.data = serde_json::to_value(comm_data).unwrap();

    let update = Communities::update(&community);

    match update {
        Ok(c) => {
            println!("Community {} updated", c.tag);
            return HttpResponse::Found().header("Location", format!("/{}/add_experience_form/{}", &lang, code)).finish()
        },
        Err(e) => {
            println!("Community update failed: {}", e);
            return e.error_response()
        }
    };
}