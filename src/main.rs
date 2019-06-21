#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use {crate::models::Raffle, std::sync::RwLock};

// Import modules
mod models;
mod routes;

fn main() {
    // Start web server...
    rocket::ignite()
        .mount(
            // off host root...
            "/",
            // with the following routes...
            routes![
                routes::create_ticket,
                routes::create_ticket_with,
                routes::get_ticket_from,
                routes::get_ticket_list,
                routes::append_to_ticket,
                routes::evaluate_ticket,
            ],
        )
        // and this internal state
        .manage(RwLock::new(Raffle::instantiate()))
        .launch();
}
