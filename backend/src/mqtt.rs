use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;

use crate::constants::mqtt::{KEEP_ALIVE_SECS, TOPIC_PATTERN};
use crate::database::pg::PayloadRepository;
use crate::database::schema::Payload;
use crate::state::app_state::AppState;

/// Hàm chạy MQTT client, lắng nghe topic TOPIC_PATTERN và cập nhật state khi có dữ liệu mới

pub async fn run_mqtt_client(
    state: Arc<AppState>,
    repo: Arc<PayloadRepository>,
    mqtt_broker: &str,
    mqtt_port: u16,
) {
    let mut mqttoptions = MqttOptions::new("rust-backend", mqtt_broker, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(KEEP_ALIVE_SECS));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe(TOPIC_PATTERN, QoS::AtMostOnce).await.unwrap();

    loop {
        if let Ok(Event::Incoming(Packet::Publish(p))) = eventloop.poll().await {
            let payload: Payload = match serde_json::from_slice::<Payload>(&p.payload) {
                Ok(v) => v,
                Err(_) => {
                    println!("Failed to parse payload: {:?}", p.payload);
                    continue;
                }
            };

            let payload = Arc::new(payload);
            let repo = repo.clone();

            state.latest_data.insert(payload.node_id.clone(), payload.clone());

            // Process payload: update data, run fire detection, update path if needed
            state.process_payload(&payload);

            tokio::spawn({
                let payload = payload.clone();
                async move {
                    if let Err(e) = repo.save_payload(&payload).await {
                        eprintln!("Failed to save payload to database: {:?}", e);
                    }
                }
            });
        }
    }
}
