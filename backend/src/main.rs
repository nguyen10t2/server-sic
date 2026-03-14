use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, get, web};
use log::info;
use std::sync::Arc;

use esp32::configs::env::ENV;
use esp32::database::pg::PayloadRepository;
use esp32::database::pool::DB;
use esp32::state::app_state::AppState;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Found().insert_header(("Location", "/api/status")).finish()
}

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

    // Bắt đầu MPSC để xử lý tác vụ ghi DB dạng queue
    let (db_tx, mut db_rx) = tokio::sync::mpsc::channel::<esp32::database::schema::Payload>(1000);

    // --- DB WORKER: Xử lý Ghi DB Batch/Queue ---
    let db_repo_clone = payload_repo.clone();
    tokio::spawn(async move {
        let mut batch = Vec::new();
        let batch_size = 50;
        let timeout_duration = std::time::Duration::from_millis(500);

        loop {
            match tokio::time::timeout(timeout_duration, db_rx.recv()).await {
                Ok(Some(payload)) => {
                    batch.push(payload);
                    if batch.len() >= batch_size {
                        if let Err(e) = db_repo_clone.save_payloads_batch(&batch).await {
                            log::error!("DB batch insert failed: {:?}", e);
                        }
                        batch.clear();
                    }
                }
                Ok(None) => break, // Channel bị đóng
                Err(_) => {
                    // Xảy ra timeout (đã qua 500ms)
                    if !batch.is_empty() {
                        if let Err(e) = db_repo_clone.save_payloads_batch(&batch).await {
                            log::error!("DB batch insert failed (on timeout): {:?}", e);
                        }
                        batch.clear();
                    }
                }
            }
        }
    });

    // Khởi tạo state chung
    let shared_state = Arc::new(AppState::new(Some(mqtt_client.clone()), db_tx));

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

            // Lấy ra các nodes bị timeout để xử lý riêng
            let mut dead_payloads = Vec::new();
            for mut entry in watchdog_state.latest_data.iter_mut() {
                let payload = Arc::make_mut(entry.value_mut());
                if current_time - payload.timestamp > timeout_threshold {
                    if payload.status != 3 {
                        // status == 3 indicates NODEDEAD
                        info!(
                            "CẢNH BÁO: Node {} mất kết nối! Đang đánh dấu DEAD.",
                            payload.node_id
                        );
                        payload.status = 3; // Mark as NODEDEAD
                        dead_payloads.push(payload.clone());
                    }
                }
            }

            // Xử lý payloads mà không giữ lock iter_mut() trên DashMap
            for payload in dead_payloads {
                watchdog_state.process_payload(&payload);
            }
        }
    });

    // Chạy MQTT ở background
    let mqtt_state = shared_state.clone();
    tokio::spawn(async move {
        esp32::mqtt::run_mqtt_client(mqtt_state, mqtt_client, eventloop).await;
    });

    // Chạy Web Server
    let app_data = web::Data::from(shared_state);

    info!("Server running at http://{}:{}/api/status", ip, port);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default())
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .service(index)
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
