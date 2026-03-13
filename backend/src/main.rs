use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use log::info;
use std::sync::Arc;

use esp32::configs::env::ENV;
use esp32::database::pg::PayloadRepository;
use esp32::database::pool::DB;
use esp32::state::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok(); // Load .env file

    // Logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let ip = ENV.ip.clone();
    let port = ENV.port;

    let mqtt_broker = ENV.mqtt_broker.clone();
    let mqtt_port = ENV.mqtt_port;

    let payload_repo = Arc::new(PayloadRepository::new(DB.clone()));

    // Khởi tạo MQTT client
    use rumqttc::{AsyncClient, MqttOptions};
    use std::time::Duration;
    let mut mqttoptions = MqttOptions::new("rust-backend", mqtt_broker, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(esp32::constants::mqtt::KEEP_ALIVE_SECS));
    let (mqtt_client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // Khởi tạo state chung
    let shared_state = Arc::new(AppState::new(Some(mqtt_client.clone())));

    // --- WATCHDOG: Kiểm tra Node mất mạng/chết ---
    let watchdog_state = shared_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            let timeout_threshold = 15_000; // 15 giây

            for mut entry in watchdog_state.latest_data.iter_mut() {
                let payload = Arc::make_mut(entry.value_mut());
                if current_time - payload.timestamp > timeout_threshold {
                    if payload.status != 3 {
                        info!(
                            "CẢNH BÁO: Node {} mất kết nối! Đang đánh dấu DEAD.",
                            payload.node_id
                        );
                        payload.status = 3; // NODEDEAD
                        // Ta có thể gọi process_payload để update lại đồ thị và path,
                        // nhưng gọi process_payload đòi hỏi &Payload, ta fake 1 lần xử lý:
                        watchdog_state.process_payload(payload);
                    }
                }
            }
        }
    });

    // Chạy MQTT ở background
    let mqtt_state = shared_state.clone();
    tokio::spawn(async move {
        esp32::mqtt::run_mqtt_client(mqtt_state, payload_repo, mqtt_client, eventloop).await;
    });

    // Chạy Web Server
    let app_data = web::Data::from(shared_state);

    info!("Server running at http://{}:{}/api/status", ip, port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
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
