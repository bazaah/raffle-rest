use {
    crate::models::Raffle,
    rocket::{http::Status, State},
    rocket_contrib::json::Json,
    serde_json::value::Value as jVal,
    std::sync::RwLock,
};

type Internal<'r> = State<'r, RwLock<Raffle>>;

#[post("/ticket")]
pub fn create_ticket(state: Internal) -> Result<Respond, Status> {
    match state.write() {
        Ok(mut raffle) => {
            let ticket_id = raffle.new_ticket(None);
            Ok(Respond::Ok(format!(
                "Added ticket <{}> with [10] lines",
                ticket_id
            )))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/ticket/<lines>")]
pub fn create_ticket_with(state: Internal, lines: u64) -> Respond {
    match state.write() {
        Ok(mut raffle) => {
            let ticket_id = raffle.new_ticket(Some(lines));
            Respond::Ok(format!(
                "Added ticket <{}> with [{}] lines",
                ticket_id, lines
            ))
        }
        Err(e) => Respond::LockPoisoned(format!("{}", e)),
    }
}

#[get("/ticket")]
pub fn get_ticket_list(state: Internal) -> Respond {
    match state.read() {
        Ok(raffle) => Respond::Success(Json(raffle.get_ticket_list())),
        Err(e) => Respond::LockPoisoned(format!("{}", e)),
    }
}

#[get("/ticket/<id>")]
pub fn get_ticket_from(state: Internal, id: u64) -> Respond {
    match state.read() {
        Ok(raffle) => match raffle.get_ticket(id) {
            Ok(ticket) => Respond::Success(Json(ticket)),
            Err(e) => Respond::Unprocessable(format!("{}", e)),
        },
        Err(e) => Respond::LockPoisoned(format!("{}", e)),
    }
}

#[put("/ticket/<id>?<append>")]
pub fn append_to_ticket(state: Internal, id: u64, append: Option<u64>) -> Respond {
    match (append, state.write()) {
        (Some(lines), Ok(mut raffle)) => match raffle.append_ticket(id, lines) {
            Ok(_) => Respond::Ok(format!("Appended [{}] lines to ticket <{}>", lines, id)),
            Err(e) => Respond::Unprocessable(format!("{}", e)),
        },
        (None, _) => Respond::BadRequest(format!("Malformed query: [append={{unsigned integer}}]")),
        (_, Err(e)) => Respond::LockPoisoned(format!("{}", e)),
    }
}

#[put("/eval/<id>")]
pub fn evaluate_ticket(state: Internal, id: u64) -> Respond {
    match state.write() {
        Ok(mut raffle) => match raffle.evaluate_ticket(id) {
            Ok(ticket) => Respond::Success(Json(ticket)),
            Err(e) => Respond::Unprocessable(format!("{}", e)),
        },
        Err(e) => Respond::LockPoisoned(format!("{}", e)),
    }
}

#[derive(Responder)]
pub enum Respond {
    #[response(status = 200)]
    Ok(String),
    #[response(status = 200, content_type = "json")]
    Success(Json<jVal>),
    #[response(status = 422)]
    Unprocessable(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    LockPoisoned(String),
}
