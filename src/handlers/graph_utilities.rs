use serde::{Serialize, Deserialize};
use std::fmt;

use bigdecimal::{ToPrimitive};

use crate::models::{Lenses, Nodes, People};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CytoGraph {
    pub nodes: Vec<CytoNode>,
    pub edges: Vec<CytoEdge>,
}

impl CytoGraph {
    pub fn add_person(p: &People) -> GNode {
        let person_node = GNode {
            id: format!("P-{}", p.id),
            node_type: String::from("Person"),
            text: vec![format!("{}", p.date_created)],
            shape: String::from("ellipse"),
            size: 25,
            color: String::from("orange"),
            inclusivity: 0.0,
            href: format!("/person_network_graph/{}", p.id),
        };

        person_node
    }

    pub fn add_node(n : &Nodes) -> GNode {
        let (colour, shape): (String, String) = if n.domain_token == "person" {
            (String::from("green"), String::from("rectangle"))
        } else {
            (String::from("blue"), String::from("triangle"))
        };

        let node = GNode {
            id: format!("{}", &n.node_name),
            node_type: String::from("Node"),
            text: vec![n.domain_token.to_owned()],
            shape: shape,
            size: 25,
            color: colour,
            inclusivity: 0.0,
            href: format!("/node/{}", n.node_name),
        };

        node
    }

    pub fn add_lens_edge(l: &Lenses) -> GEdge {
        let edge = GEdge {
            id: format!("P{}-{}", &l.person_id, &l.node_name),
            source: format!("P-{}", &l.person_id),
            target: format!("{}", &l.node_name),
            text: l.statements.to_owned(),
            weight: l.inclusivity.to_f32().expect("unable to convert decimal"),
        };

        edge
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CytoNode {
    pub data: GNode,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CytoEdge {
    pub data: GEdge,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub text: Vec<String>,
    pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub struct GNode {
    pub id: String,
    pub node_type: String,
    pub text: Vec<String>,
    pub shape: String,
    pub size: i32,
    pub color: String,
    pub inclusivity: f32,
    pub href: String,
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({})", self.id, self.node_type, self.inclusivity)
    }
}

/// Accepts vectors of people, nodes and lenses and formats the data into
/// JSON acceptable to Cytoscape.js
pub fn generate_cyto_graph(
    people_vec: Vec<People>,
    node_vec: Vec<Nodes>,
    lens_vec: Vec<Lenses>
) -> CytoGraph {

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    for p in people_vec {

        let ni = CytoGraph::add_person(&p);

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for n in node_vec {

        let ni = CytoGraph::add_node(&n);

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for l in lens_vec {

        let edge = CytoGraph::add_lens_edge(&l);

        cyto_edge_array.push(CytoEdge {
            data: edge,
        });
    };

    let graph: CytoGraph = CytoGraph {
        nodes: cyto_node_array,
        edges: cyto_edge_array,
    };

    graph
}

pub fn generate_node_cyto_graph(
    node_vec: Vec<Nodes>,
    lens_vec: Vec<Lenses>
) -> CytoGraph {
    // reconfigure this to connect nodes to each other

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    for n in node_vec {

        let ni = CytoGraph::add_node(&n);

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for l in lens_vec {

        let edge = CytoGraph::add_lens_edge(&l);

        cyto_edge_array.push(CytoEdge {
            data: edge,
        });
    };

    let graph: CytoGraph = CytoGraph {
        nodes: cyto_node_array,
        edges: cyto_edge_array,
    };

    graph
}