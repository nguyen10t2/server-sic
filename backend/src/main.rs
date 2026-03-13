
use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use esp32::configs::env::ENV;
use esp32::database::pg::PayloadRepository;
use esp32::database::pool::DB;
use esp32::state::app_state::AppState;

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
        esp32::mqtt::run_mqtt_client(mqtt_state, payload_repo, &mqtt_broker, mqtt_port).await;
    });

    // Chạy Web Server
    let app_data = web::Data::from(shared_state);

    println!("Server running at http://{}:{}/api/status", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(esp32::controllers::api::ws_index)
            .service(esp32::controllers::api::get_status)
            .service(esp32::controllers::api::get_fire_status)
            .service(esp32::controllers::api::get_all_evacuation_paths)
            .service(esp32::controllers::api::get_evacuation_path)
            .service(esp32::controllers::api::get_building_graph)
    })
    .bind((ip, port))?
    .run()
    .await
}
