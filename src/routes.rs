use {
    crate::models::Raffle,
    rocket::{
        http::Status,
        request::Request,
        response::{self, Responder, Response as rResponse},
        State,
    },
    rocket_contrib::json::Json,
    serde_json::value::Value as jVal,
    std::sync::RwLock,
};

type Internal<'r> = State<'r, RwLock<Raffle>>;
type Response = Result<Good, Fail>;

#[post("/ticket")]
pub fn create_ticket(state: Internal) -> Response {
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

#[post("/ticket/<lines>")]
pub fn create_ticket_with(state: Internal, lines: u64) -> Response {
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

#[get("/ticket")]
pub fn get_ticket_list(state: Internal) -> Response {
    match state.read() {
        Ok(raffle) => Ok(Good::Success(Json(raffle.get_ticket_list()))),
        Err(_) => Err(Fail::LockPoisoned),
    }
}

#[get("/ticket/<id>")]
pub fn get_ticket_from(state: Internal, id: u64) -> Response {
    match state.read() {
        Ok(raffle) => match raffle.get_ticket(id) {
            Ok(ticket) => Ok(Good::Success(Json(ticket))),
            Err(e) => Err(Fail::Unprocessable(format!("{}", e))),
        },
        Err(_) => Err(Fail::LockPoisoned),
    }
}

#[put("/ticket/<id>?<append>")]
pub fn append_to_ticket(state: Internal, id: u64, append: Option<u64>) -> Response {
    match (append, state.write()) {
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

#[put("/eval/<id>")]
pub fn evaluate_ticket(state: Internal, id: u64) -> Response {
    match state.write() {
        Ok(mut raffle) => match raffle.evaluate_ticket(id) {
            Ok(ticket) => Ok(Good::Success(Json(ticket))),
            Err(e) => Err(Fail::Unprocessable(format!("{}", e))),
        },
        Err(_) => Err(Fail::LockPoisoned),
    }
}

#[derive(Responder)]
pub enum Good {
    #[response(status = 200)]
    Info(String),
    #[response(status = 200, content_type = "json")]
    Success(Json<jVal>),
}

#[derive(Debug)]
pub enum Fail {
    Unprocessable(String),
    BadRequest,
    LockPoisoned,
}

impl<'r> Responder<'r> for Fail {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match self {
            Fail::Unprocessable(err) => rResponse::build()
                .sized_body(std::io::Cursor::new(err))
                .status(Status::UnprocessableEntity)
                .ok(),
            Fail::BadRequest => rResponse::build()
                .sized_body(std::io::Cursor::new(format!(
                    "Malformed query: [append={{unsigned integer}}]"
                )))
                .status(Status::BadRequest)
                .ok(),
            Fail::LockPoisoned => rResponse::build()
                .sized_body(std::io::Cursor::new(format!(
                    "Unrecoverable error: Internal state poisoned, restart the server"
                )))
                .status(Status::ServiceUnavailable)
                .ok(),
        }
    }
}
