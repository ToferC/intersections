#[macro_use]
use serde::{Serialize, Deserialize};

use crate::schema::nodes;
use super::lens::Lens;

#[derive(Serialize, Deserialize, Debug, Clone, Associations, Insertable, Queryable, PartialEq)]
#[table_name = "nodes"]
pub struct Node {
    id: i64,
    name: String,
    domain_token: String,
}