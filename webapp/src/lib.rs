#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use tera::Tera;

pub mod models;
pub mod handlers;
pub mod schema;

pub struct AppData {
    pub tmpl: Tera,
}