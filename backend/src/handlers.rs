use actix_web::{get, web, HttpResponse, Responder};
use crate::state::AppState;

#[get("/api/status")]
pub async fn get_status(data: web::Data<AppState>) -> impl Responder {
    let current_val = data.latest_data.lock().unwrap();
    HttpResponse::Ok().json(&*current_val)
}