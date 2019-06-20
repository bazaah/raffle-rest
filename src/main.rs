#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use {
    rand::{
        distributions::{Distribution, Uniform},
        thread_rng as rng,
    },
    rocket::{http::Status, response::status, State},
    rocket_contrib::json::Json,
    serde::Serialize,
    serde_json::{json, value::Value as jVal},
    std::{fmt, sync::RwLock},
};

#[post("/ticket")]
fn create_ticket(state: State<RwLock<Raffle>>) -> Result<CrateRespond, Status> {
    match state.write() {
        Ok(mut raffle) => match raffle.new_ticket() {
            Ok(_) => Ok(CrateRespond::Ok(format!("Added ticket with [10] lines"))),
            Err(_) => Err(Status::InsufficientStorage),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/ticket/<lines>")]
fn create_ticket_with(state: State<RwLock<Raffle>>, lines: u64) -> Result<CrateRespond, Status> {
    match state.write() {
        Ok(mut raffle) => match raffle.new_ticket_from(lines) {
            Ok(_) => Ok(CrateRespond::Ok(format!(
                "Added ticket with [{}] lines",
                lines
            ))),
            Err(_) => Err(Status::InsufficientStorage),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/ticket")]
fn get_ticket_list(state: State<RwLock<Raffle>>) -> Result<CrateRespond, Status> {
    match state.read() {
        Ok(raffle) => Ok(CrateRespond::Success(Json(raffle.get_ticket_list()))),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/ticket/<id>")]
fn get_ticket(id: u64) {}

#[put("/ticket/<id>")]
fn append_to_ticket(id: u64) {}

#[put("/eval/<id>")]
fn evaluate_ticket(id: u64) {}

#[derive(Responder)]
enum CrateRespond {
    #[response(status = 200)]
    Ok(String),
    #[response(status = 200, content_type = "json")]
    Success(Json<Vec<jVal>>),
    #[response(status = 507)]
    RaffleFull(String),
    #[response(status = 500)]
    LockPoisoned(String),
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                create_ticket,
                create_ticket_with,
                get_ticket,
                get_ticket_list,
                append_to_ticket,
                evaluate_ticket
            ],
        )
        .manage(RwLock::new(Raffle::instantiate()))
        .launch();
}

struct Raffle {
    tickets: Vec<Ticket>,
}

impl Raffle {
    fn instantiate() -> Self {
        let tickets: Vec<Ticket> = Vec::new();
        Raffle { tickets }
    }

    fn new_ticket(&mut self) -> Result<(), ()> {
        match self.tickets.len() == usize::max_value() {
            false => {
                self.tickets.push(Ticket::new());
                Ok(())
            }
            true => Err(()),
        }
    }

    fn new_ticket_from(&mut self, lines: u64) -> Result<(), ()> {
        match self.tickets.len() == usize::max_value() {
            false => {
                self.tickets.push(Ticket::from(lines));
                Ok(())
            }
            true => Err(()),
        }
    }

    fn get_ticket_list(&self) -> Vec<jVal> {
        let json: Vec<jVal> = self
            .tickets
            .iter()
            .enumerate()
            .map(|(idx, ticket)| {
                json!({
                    "id": idx,
                    "lines": ticket.eval_list()
                })
            })
            .collect();

        json
    }
}

#[derive(Clone, Serialize)]
struct Ticket {
    line_list: Vec<Line>,
}

impl Ticket {
    fn new() -> Self {
        let line_list = (0..10)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|rand| Line::from(rand))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    fn from(lines: u64) -> Self {
        let line_list = (0..lines)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|seed| Line::from(seed))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    fn append(&mut self, additional: u64) {
        (0..additional)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .for_each(|seed| self.line_list.push(Line::from(seed)))
    }

    fn eval_list(&self) -> Vec<u8> {
        self.line_list
            .iter()
            .inspect(|line| eprintln!("{:?}", line))
            .map(|line| line.eval_line())
            .collect::<Vec<u8>>()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Line(u8, u8, u8);

impl Line {
    fn from((x, y, z): (u8, u8, u8)) -> Self {
        Line(x, y, z)
    }

    fn eval_line(&self) -> u8 {
        match (self.0, self.1, self.2) {
            // Ordered by rule priority
            // Avoids situations where ex. (2,0,0) satisfies
            // 2 rules: |x+y+z == 2| & |x!=y && x!=z|
            (x, y, z) if x + y + z == 2 => 10,
            (x, y, z) if x == y && y == z => 5,
            (x, y, z) if x != y && x != z => 1,
            (_, _, _) => 0,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|{}|", self.eval_line())
    }
}
