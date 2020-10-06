use serde::{Serialize, Deserialize};
use diesel::prelude::*;

use crate::error_handler::CustomError;
use crate::database;

use crate::schema::nodes;

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "nodes"]
pub struct Node {
    node_name: String,
    domain_token: String,
}

impl Node {
    pub fn new(name: String, domain: String) -> Self {
        Node {
            node_name: name,
            domain_token: domain,
        }
    }

    pub fn from(node: &Node) -> Node {
        Node {
            node_name: node.node_name.to_owned(),
            domain_token: node.domain_token.to_owned(), 
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "nodes"]
pub struct Nodes {
    id: i32,
    node_name: String,
    domain_token: String,
}

impl Nodes {
    pub fn create(node: &Node) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let n = Node::from(node);
        let n = diesel::insert_into(nodes::table)
            .values(n)
            .get_results(&conn)?;
        Ok(n)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let nodes = nodes::table.load::<Nodes>(&conn)?;
        Ok(nodes)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let node = nodes::table.filter(nodes::id.eq(id)).first(&conn)?;
        Ok(node)
    }

    pub fn update(id: i32, node: &Node) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let node = diesel::update(nodes::table)
            .filter(nodes::id.eq(id))
            .set(node)
            .get_results(&conn)?;
        Ok(node)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(nodes::table.filter(nodes::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}