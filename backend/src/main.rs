mod configs;
mod controllers;
mod database;
mod mqtt;
mod state;
mod common;
mod constants;

use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use crate::configs::env::ENV;
use crate::database::pg::PayloadRepository;
use crate::database::pool::DB;
use crate::state::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok(); // Load .env file

    let ip = ENV.ip.clone();
    let port = ENV.port;

    let mqtt_broker = ENV.mqtt_broker.clone();
    let mqtt_port = ENV.mqtt_port;

    let payload_repo = Arc::new(PayloadRepository::new(DB.clone()));

    // Khởi tạo state chung
    let shared_state = Arc::new(AppState::new());

    // Chạy MQTT ở background
    let mqtt_state = shared_state.clone();
    tokio::spawn(async move {
        mqtt::run_mqtt_client(mqtt_state, payload_repo, &mqtt_broker, mqtt_port).await;
    });

    // Chạy Web Server
    let app_data = web::Data::from(shared_state);

    println!("Server running at http://{}:{}/api/status", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(controllers::api::get_status)
            .service(controllers::api::get_fire_status)
            .service(controllers::api::get_evacuation_path)
            .service(controllers::api::get_all_evacuation_paths)
            .service(controllers::api::get_building_graph)
    })
    .bind((ip, port))?
    .run()
    .await
}
