use {
    crate::models::Raffle,
    rocket::{
        http::{ContentType, Status},
        request::Request,
        response::{self, Responder, Response as rResponse},
        Rocket, State,
    },
    serde_json::{json, value::Value as jVal},
    std::{io::Cursor, sync::RwLock},
};

pub fn rocket() -> Rocket {
    // Start web server...
    rocket::ignite()
        .mount(
            // off host root...
            "/",
            // with the following routes...
            routes![
                create_ticket,
                create_ticket_with,
                get_ticket_from,
                get_ticket_list,
                append_to_ticket,
                evaluate_ticket,
            ],
        )
        // and this internal state
        .manage(RwLock::new(Raffle::instantiate()))
}

// Aliases for easier readability
type Internal<'r> = State<'r, RwLock<Raffle>>;
type Response = Result<Good, Fail>;

// Creates a new ticket with the default number of Lines [10]
#[get("/ticket")]
fn create_ticket(state: Internal) -> Response {
    match state.write() {
        Ok(mut raffle) => {
            let ticket_id = raffle.new_ticket(None);
            Ok(Good::Info(format!(
                "Added ticket <{}> with [10] lines",
                ticket_id
            )))
        }
        Err(_) => Err(Fail::LockPoisoned),
    }
}

// Creates a new ticket with a user defined number of Lines [lines]
#[get("/ticket/<lines>")]
fn create_ticket_with(state: Internal, lines: u64) -> Response {
    match state.write() {
        Ok(mut raffle) => {
            let ticket_id = raffle.new_ticket(Some(lines));
            Ok(Good::Info(format!(
                "Added ticket <{}> with [{}] lines",
                ticket_id, lines
            )))
        }
        Err(_) => Err(Fail::LockPoisoned),
    }
}

// Returns the entire list of Tickets as Json
#[get("/ticket/list")]
fn get_ticket_list(state: Internal) -> Response {
    match state.read() {
        Ok(raffle) => Ok(Good::Success(raffle.get_ticket_list())),
        Err(_) => Err(Fail::LockPoisoned),
    }
}

// Returns a user defined Ticket via its id [id]
#[get("/ticket/list/<id>")]
fn get_ticket_from(state: Internal, id: u64) -> Response {
    match state.read() {
        Ok(raffle) => match raffle.get_ticket(id) {
            Ok(ticket) => Ok(Good::Success(ticket)),
            Err(e) => Err(Fail::Unprocessable(format!("{}", e))),
        },
        Err(_) => Err(Fail::LockPoisoned),
    }
}

// Appends a user defined number of Lines [append] to a Ticket via its id [id]
#[get("/ticket/append/<id>?<lines>")]
fn append_to_ticket(state: Internal, id: u64, lines: Option<u64>) -> Response {
    match (lines, state.write()) {
        (Some(lines), Ok(mut raffle)) => match raffle.append_ticket(id, lines) {
            Ok(_) => Ok(Good::Info(format!(
                "Appended [{}] lines to ticket <{}>",
                lines, id
            ))),
            Err(e) => Err(Fail::Unprocessable(format!("{}", e))),
        },
        (None, _) => Err(Fail::BadRequest),
        (_, Err(_)) => Err(Fail::LockPoisoned),
    }
}

// Uses up a Ticket via its id [id] and returns its score
#[get("/eval/<id>")]
fn evaluate_ticket(state: Internal, id: u64) -> Response {
    match state.write() {
        Ok(mut raffle) => match raffle.evaluate_ticket(id) {
            Ok(ticket) => Ok(Good::Success(ticket)),
            Err(e) => Err(Fail::Unprocessable(format!("{}", e))),
        },
        Err(_) => Err(Fail::LockPoisoned),
    }
}

// Successful responses
#[derive(Debug)]
enum Good {
    Info(String),
    Success(jVal),
}

// Custom implementation for API responses
impl<'r> Responder<'r> for Good {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match self {
            Good::Info(i) => rResponse::build()
                .sized_body(Cursor::new(json!({"code": 200, "info": i}).to_string()))
                .header(ContentType::JSON)
                .status(Status::Ok)
                .ok(),
            Good::Success(value) => rResponse::build()
                .sized_body(Cursor::new(json!({"code": 200, "data": value}).to_string()))
                .header(ContentType::JSON)
                .status(Status::Ok)
                .ok(),
        }
    }
}

// Errored responses
#[derive(Debug)]
enum Fail {
    Unprocessable(String),
    BadRequest,
    LockPoisoned,
}

// Custom implementation for API specific errors
impl<'r> Responder<'r> for Fail {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match self {
            Fail::Unprocessable(err) => rResponse::build()
                .sized_body(Cursor::new(json!({"code": 422, "info": err}).to_string()))
                .header(ContentType::JSON)
                .status(Status::UnprocessableEntity)
                .ok(),
            Fail::BadRequest => rResponse::build()
                .sized_body(Cursor::new(json!({"code": 400, "info": "malformed query: [append={{unsigned integer}}]"}).to_string()))
                .header(ContentType::JSON)
                .status(Status::BadRequest)
                .ok(),
            Fail::LockPoisoned => rResponse::build()
                .sized_body(Cursor::new(json!({"code": 503, "info": "Unrecoverable error: Internal state poisoned, restart the server"}).to_string()))
                .header(ContentType::JSON)
                .status(Status::ServiceUnavailable)
                .ok(),
        }
    }
}

/*
Code
-------------------------------------------------------------------------------
Tests
*/

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::Client;
    use serde_json::json;

    #[test]
    fn Route_create_ticket() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/ticket").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 200, "info": "Added ticket <1> with [10] lines"}).to_string())
        );
    }

    #[test]
    fn Route_create_ticket_with() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/ticket/5").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 200, "info": "Added ticket <1> with [5] lines"}).to_string())
        );
    }

    #[test]
    fn Route_get_ticket_list() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/ticket/list").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 200, "data": []}).to_string())
        );
    }

    #[test]
    fn Route_get_ticket_from_failure() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/ticket/list/1").dispatch();
        assert_eq!(response.status(), Status::UnprocessableEntity);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 422, "info": "Ticket id: 1 doesn't exist"}).to_string())
        );
    }

    #[test]
    fn Route_append_to_ticket_failure() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/ticket/append/1?lines=10").dispatch();
        assert_eq!(response.status(), Status::UnprocessableEntity);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 422, "info": "Ticket id: 1 doesn't exist"}).to_string())
        );
    }

    #[test]
    fn Route_evaluate_ticket_failure() {
        let client = Client::new(rocket()).expect("Valid rocket instance");
        let mut response = client.get("/eval/1").dispatch();
        assert_eq!(response.status(), Status::UnprocessableEntity);
        assert_eq!(
            response.body_string(),
            Some(json!({"code": 422, "info": "Ticket id: 1 doesn't exist"}).to_string())
        );
    }
}
