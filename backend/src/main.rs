mod state;
mod mqtt;
mod handlers;
mod helper;

use actix_web::{web, App, HttpServer};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::{helper::get_env, state::AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok(); // Load .env file

    let ip = get_env("ip", Some("localhost")).unwrap();
    let port = get_env("port", Some("8080")).unwrap().parse::<u16>().unwrap();

    let mqtt_broker = get_env("mqtt_broker", Some("localhost")).unwrap();
    let mqtt_port = get_env("mqtt_port", Some("1883")).unwrap().parse::<u16>().unwrap();

    // Khởi tạo state chung
    let shared_state = Arc::new(AppState {
        latest_data: Mutex::new(HashMap::new()),
    });

    // Chạy MQTT ở background
    let mqtt_state = shared_state.clone();
    tokio::spawn(async move {
        mqtt::run_mqtt_client(mqtt_state, &mqtt_broker, mqtt_port).await;
    });

    // Chạy Web Server
    let app_data = web::Data::from(shared_state);

    println!("Server running at http://{}:{}/api/status", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(handlers::get_status)
    })
    .bind((ip, port))?
    .run()
    .await
}