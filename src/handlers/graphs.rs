use actix_web::{web, get, HttpResponse, Responder};
use crate::AppData;
use tera::{Context};

use crate::models::{Lenses, Nodes, People};

use crate::handlers::{generate_cyto_graph};

#[get("/full_network_graph")]
pub async fn full_network_graph(
    data: web::Data<AppData>
) -> impl Responder {
        
    let people_vec = People::find_all().expect("Unable to load people");
    
    let lens_vec = Lenses::find_all().expect("Unable to load lenses");

    let node_vec = Nodes::find_all().expect("Unable to load nodes");

    let graph = generate_cyto_graph(people_vec, node_vec, lens_vec);

    let j = serde_json::to_string_pretty(&graph).unwrap();
    
    let mut ctx = Context::new();
    ctx.insert("graph_data", &j);

    let title = "Full Network Graph";
    ctx.insert("title", title);

    let node_names = Nodes::find_all_names().expect("Unable to load names");
    ctx.insert("node_names", &node_names);
    
    let rendered = data.tmpl.render("network_graph.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

