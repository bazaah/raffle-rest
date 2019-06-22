#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use crate::routes::rocket;

// Import modules
mod models;
mod routes;

fn main() {
    rocket().launch();
}
