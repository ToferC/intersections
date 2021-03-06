use serde::{Serialize, Deserialize};
use std::fmt;

use bigdecimal::{ToPrimitive};
use std::collections::{BTreeMap, HashMap};

use crate::models::{Lenses, Nodes, People};

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
    pub fn from_lens(l: &Lenses) -> GEdge {
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
            (String::from("green"), String::from("rectangle"))
        } else {
            (String::from("blue"), String::from("triangle"))
        };

        let href = match community {
            Some(c) => format!("/node/{}/{}", c, n.node_name),
            None => format!("/node/{}", n.node_name),
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

    pub fn from_lens(l: &Lenses, community: &Option<String>) -> GNode {

        let (colour, shape): (String, String) = if l.node_domain == "person" {
            (String::from("green"), String::from("rectangle"))
        } else {
            (String::from("blue"), String::from("triangle"))
        };

        let href = match community {
            Some(c) => format!("/node/{}/{}", c, l.node_name.trim()),
            None => format!("/node/{}", l.node_name.trim()),
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
}

/// Accepts vectors of people, nodes and lenses and formats the data into
/// JSON acceptable to Cytoscape.js
pub fn generate_cyto_graph(
    people_vec: Vec<People>,
    node_vec: Vec<Nodes>,
    lens_vec: Vec<Lenses>,
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

    for l in lens_vec {

        let edge = GEdge::from_lens(&l);

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
    lens_vec: Vec<Lenses>,
    people_connections: HashMap<i32, Vec<String>>,
    community: Option<String>,
) -> CytoGraph {
    // reconfigure this to connect nodes to each other

    let mut cyto_node_array: Vec<CytoNode> = Vec::new();
    let mut cyto_edge_array: Vec<CytoEdge> = Vec::new();

    // what else do I need to render the graph?
    let mut edge_map: BTreeMap<String, GEdge> = BTreeMap::new();

    for l in &lens_vec {

        let n = GNode::from_lens(&l, &community);

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

    println!("{:?}", &graph);

    graph
}