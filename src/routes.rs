use {
    crate::models::Raffle,
    rocket::{http::Status, State},
    rocket_contrib::json::Json,
    serde_json::value::Value as jVal,
    std::sync::RwLock,
};

#[post("/ticket")]
pub fn create_ticket(state: State<RwLock<Raffle>>) -> Result<CrateRespond, Status> {
    match state.write() {
        Ok(mut raffle) => match raffle.new_ticket() {
            Ok(_) => Ok(CrateRespond::Ok(format!("Added ticket with [10] lines"))),
            Err(_) => Err(Status::InsufficientStorage),
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/ticket/<lines>")]
pub fn create_ticket_with(
    state: State<RwLock<Raffle>>,
    lines: u64,
) -> Result<CrateRespond, Status> {
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
pub fn get_ticket_list(state: State<RwLock<Raffle>>) -> Result<CrateRespond, Status> {
    match state.read() {
        Ok(raffle) => Ok(CrateRespond::Success(Json(raffle.get_ticket_list()))),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/ticket/<id>")]
pub fn get_ticket(id: u64) {}

#[put("/ticket/<id>")]
pub fn append_to_ticket(id: u64) {}

#[put("/eval/<id>")]
pub fn evaluate_ticket(id: u64) {}

#[derive(Responder)]
pub enum CrateRespond {
    #[response(status = 200)]
    Ok(String),
    #[response(status = 200, content_type = "json")]
    Success(Json<Vec<jVal>>),
    #[response(status = 507)]
    RaffleFull(String),
    #[response(status = 500)]
    LockPoisoned(String),
}
