use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;

use crate::state::Payload;
use crate::state::AppState;

/**
 * Hàm chạy MQTT client, lắng nghe topic "fire/#" và cập nhật state khi có dữ liệu mới
 */
pub async fn run_mqtt_client(state: Arc<AppState>, mqtt_broker: &str, mqtt_port: u16) {
    let mut mqttoptions = MqttOptions::new("rust-backend", mqtt_broker, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("fire/#", QoS::AtMostOnce).await.unwrap();

    loop {
        if let Ok(Event::Incoming(Packet::Publish(p))) = eventloop.poll().await {
            let payload: Payload = match serde_json::from_slice(&p.payload) {
                Ok(v) => v,
                Err(_) => {
                    println!("Failed to parse payload: {:?}", p.payload);
                    continue;
                },
            };
            let mut data = state.latest_data.lock().unwrap();
            data.insert(payload.node_id.clone(), payload);
        }
    }
}
