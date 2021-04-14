use serde::{Serialize, Deserialize};
use std::fmt;

use bigdecimal::{ToPrimitive};
use std::collections::{BTreeMap, HashMap};
use inflector::Inflector;

use crate::models::{Experiences, Nodes, People};

use super::AggregateExperience;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CytoGraph {
    pub nodes: Vec<CytoNode>,
    pub edges: Vec<CytoEdge>,
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

impl GEdge {
    pub fn from_experience(l: &Experiences) -> GEdge {
        let edge = GEdge {
            id: format!("P{}-{}", &l.person_id, &l.node_name.trim()),
            source: format!("P-{}", &l.person_id),
            target: format!("{}", &l.node_name.trim()),
            text: l.statements.to_owned(),
            weight: l.inclusivity.to_f32().expect("unable to convert decimal"),
        };

        edge
    }
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

impl GNode {
    pub fn from_person(p: &People) -> GNode {
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

    pub fn from_node(n : &Nodes, community: &Option<String>) -> GNode {
        let (colour, shape): (String, String) = if n.domain_token == "person" {
            (String::from("green"), String::from("ellipse"))
        } else {
            (String::from("blue"), String::from("rectangle"))
        };

        let href = match community {
            Some(c) => format!("/community_node/{}/{}", c, n.slug),
            None => format!("/node/{}", n.slug),
        };

        let node = GNode {
            id: format!("{}", &n.node_name.trim()),
            node_type: String::from("Node"),
            text: vec![n.domain_token.to_owned()],
            shape: shape,
            size: 25,
            color: colour,
            inclusivity: 0.0,
            href,
        };

        node
    }

    pub fn from_experience(l: &Experiences, community: &Option<String>) -> GNode {

        let (colour, shape): (String, String) = if l.node_domain == "person" {
            (String::from("green"), String::from("ellipse"))
        } else {
            (String::from("blue"), String::from("rectangle"))
        };

        let slug = l.node_name.trim().to_string().to_snake_case();

        let href = match community {
            Some(c) => format!("/community_node/{}/{}", c, slug),
            None => format!("/node/{}", slug),
        };

        let node = GNode {
            id: format!("{}", &l.node_name.trim()),
            node_type: String::from("Node"),
            text: vec![l.node_domain.to_owned()],
            shape: shape,
            size: 25,
            color: colour,
            inclusivity: l.inclusivity.to_f32().expect("Unable to convert BigDecimal"),
            href,
        };

        node
    }

    pub fn from_agg_experience(a: &AggregateExperience, community: &Option<String>) -> GNode {

        let (colour, shape): (String, String) = if a.domain == "person" {
            (String::from("green"), String::from("ellipse"))
        } else {
            (String::from("blue"), String::from("rectangle"))
        };

        let slug = a.name.trim().to_string().to_snake_case();

        let href = match community {
            Some(c) => format!("/community_node/{}/{}", c, slug),
            None => format!("/node/{}", slug),
        };

        let node = GNode {
            id: format!("{}", &a.name.trim()),
            node_type: String::from("Node"),
            text: vec![a.domain.to_owned()],
            shape: shape,
            size: 25,
            color: colour,
            inclusivity: a.mean_inclusivity.to_f32().expect("Unable to convert BigDecimal"),
            href,
        };

        node
    }
}

/// Accepts vectors of people, nodes and experiences and formats the data into
/// JSON acceptable to Cytoscape.js
pub fn generate_cyto_graph(
    people_vec: Vec<People>,
    node_vec: Vec<Nodes>,
    experience_vec: Vec<Experiences>,
    community: Option<String>,
) -> CytoGraph {

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    for p in people_vec {

        let ni = GNode::from_person(&p);

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for n in node_vec {

        let ni = GNode::from_node(&n, &community);

        cyto_node_array.push(CytoNode {
            data: ni,
        });
    };

    for l in experience_vec {

        let edge = GEdge::from_experience(&l);

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
    experience_vec: Vec<Experiences>,
    people_connections: HashMap<i32, Vec<String>>,
    community: Option<String>,
) -> CytoGraph {
    // reconfigure this to connect nodes to each other

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    // what else do I need to render the graph?
    let mut edge_map: BTreeMap<String, GEdge> = BTreeMap::new();

    // prep for aggregating experiences
    let mut node_id_vec: Vec<i32> = Vec::new();

    for l in &experience_vec {
        node_id_vec.push(l.node_id);
    };

    node_id_vec.sort();
    node_id_vec.dedup();

    let mut aggregate_experiences: Vec<AggregateExperience> = Vec::new();

    for i in node_id_vec {
        let mut temp_experience_vec: Vec<Experiences> = Vec::new();

        for l in &experience_vec {

            if i == l.node_id {
                temp_experience_vec.push(l.clone());
            }
            // count people associated to multiple similar nodes
            // show connections across the nodes and experiences
        };

        if temp_experience_vec.len() > 0 {
            let agg_experiences = AggregateExperience::from(temp_experience_vec);
            aggregate_experiences.push(agg_experiences);
        }
    };

    aggregate_experiences.sort_by(|a, b|b.count.partial_cmp(&a.count).unwrap());
    aggregate_experiences.dedup();

    for a in &aggregate_experiences {

        let n = GNode::from_agg_experience(&a, &community);

        cyto_node_array.push(CytoNode {
            data: n,
        });
    };

    for (_k, v) in &people_connections {

        let copy_vec = v.clone();

        for n in v.into_iter() {
            let n = n.trim();

            for copy_n in &copy_vec {
                let copy_n = copy_n.trim();

                if n != copy_n {

                    let id = format!("{}-{}", n, copy_n);
                    let inverse_id = format!("{}-{}", copy_n, n);

                    let mut exists: usize = 0; // map 1 to id, 2 to inverse_id

                    // figure out which mapping already exists
                    if edge_map.contains_key(&id) {
                        exists = 1;
                    };

                    if edge_map.contains_key(&inverse_id) {
                        exists = 2;
                    }

                    // increment
                    let target = match exists {
                        1 => id,
                        2 => inverse_id,
                        _ => id,
                    };

                    if exists != 0 {
                        // edge already exists, increment
                        let e = edge_map.get_mut(&target).expect("Unable to map to edge");
                        e.weight += 1.0;

                    } else {
                        // no edge, create a new one
                        let edge = GEdge {
                            id: target,
                            source: format!("{}", n),
                            target: format!("{}", copy_n),
                            text: vec![String::from("")],
                            weight: 1.0,
                        };

                        edge_map.insert(edge.id.to_owned(), edge);
                    };
                }
            };
        }
    };

    for (_k, mut v) in edge_map {

        v.weight = v.weight / 2.0;
        cyto_edge_array.push(CytoEdge {
            data: v,
        });
    }

    // use people id's as bridge to create node to note connections

    let graph: CytoGraph = CytoGraph {
        nodes: cyto_node_array,
        edges: cyto_edge_array,
    };

    graph
}