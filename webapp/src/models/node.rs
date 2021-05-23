use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use inflector::Inflector;

use error_handler::error_handler::CustomError;
use database;
use crate::models::{Experiences, Phrases};

use crate::schema::{nodes, phrases};

#[derive(Debug, Serialize, Deserialize, AsChangeset, Insertable)]
#[table_name = "nodes"]
pub struct Node {
    pub node_name: i32,
    pub domain_token: String,
    pub slug: String,
}

impl Node {
    pub fn new(node_name: i32, entry_name: String, domain: String) -> Self {
        Node {
            node_name,
            domain_token: domain,
            slug: entry_name.to_snake_case(),
        }
    }

    pub fn from(node: &Node) -> Node {
        Node {
            node_name: node.node_name,
            domain_token: node.domain_token.to_owned(),
            slug: node.slug.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Associations, Identifiable)]
#[table_name = "nodes"]
pub struct Nodes {
    pub id: i32,
    pub node_name: i32,
    pub domain_token: String,
    pub translation: String,
    pub synonyms: Vec<String>,
    pub slug: String,
}

impl Nodes {
    pub fn create(node: &Node) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let n = Node::from(node);
        let n = diesel::insert_into(nodes::table)
            .values(n)
            .get_result(&conn)?;
        Ok(n)
    }

    pub fn detailed_create(node: &Nodes) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let node = diesel::insert_into(nodes::table)
            .values(node)
            .get_result(&conn)?;
        Ok(node)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let nodes = nodes::table.load::<Nodes>(&conn)?;
        Ok(nodes)
    }

    pub fn find_all_names(lang: &str) -> Result<Vec<String>, CustomError> {
        let conn = database::connection()?;
        let names = nodes::table.inner_join(phrases::table
            .on(nodes::node_name.eq(phrases::id)
            .and(phrases::lang.eq(lang))))
            .select(phrases::text)
            .load::<String>(&conn)?;

        Ok(names)
    }

    /*
        experience::tables.inner_join(phrases::table
        .on(experience::node_name.eq(phrases::id)
        .and(phrases::language.eq("foo"))
        .select((all, the, columns))
    */

    pub fn find_all_linked_names_slugs(lang: &str) -> Result<Vec<(String, String)>, CustomError> {
        // return string and slug for all nodes created outside of demo community
        
        let conn = database::connection()?;

        let node_ids = Experiences::find_real_node_ids().expect("Unable to load experiences");

        let names = nodes::table.inner_join(phrases::table
            .on(nodes::node_name.eq(phrases::id)
            .and(phrases::lang.eq(lang))))
            .filter(nodes::id.eq_any(node_ids))
            .select((phrases::text, nodes::slug))
            .load::<(String, String)>(&conn)?;

        Ok(names)
    }

    pub fn find(id: i32, lang: &str) -> Result<(Self, Phrases), CustomError> {
        let conn = database::connection()?;
        let node = nodes::table.inner_join(phrases::table
            .on(nodes::node_name.eq(phrases::id)
            .and(phrases::lang.eq(lang))))
            .filter(nodes::id.eq(id))
            .first::<(Nodes, Phrases)>(&conn)?;
        Ok(node)
    }

    pub fn find_by_slug(node_slug: &String) -> Result<Self, CustomError> {
        // returns a node and localized name based on a slug and language call
        // note that the slug is not localized
        let conn = database::connection()?;
        let node = nodes::table
            .filter(nodes::slug.eq(node_slug))
            .first::<Nodes>(&conn)?;
        Ok(node)
    }

    pub fn update(id: i32, node: &Node) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let node = diesel::update(nodes::table)
            .filter(nodes::id.eq(id))
            .set(node)
            .get_result(&conn)?;
        Ok(node)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(nodes::table.filter(nodes::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}