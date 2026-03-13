use log::{error, info};
use rumqttc::{AsyncClient, Event, Packet, QoS};
use std::sync::Arc;

use crate::constants::mqtt::TOPIC_PATTERN;
use crate::database::schema::Payload;
use crate::state::app_state::AppState;

/// Hàm chạy MQTT client, lắng nghe topic TOPIC_PATTERN và cập nhật state khi có dữ liệu mới

pub async fn run_mqtt_client(
    state: Arc<AppState>,
    client: AsyncClient,
    mut eventloop: rumqttc::EventLoop,
) {
    client.subscribe(TOPIC_PATTERN, QoS::AtMostOnce).await.unwrap();

    loop {
        if let Ok(Event::Incoming(Packet::Publish(p))) = eventloop.poll().await {
            let mut payload: Payload = match serde_json::from_slice::<Payload>(&p.payload) {
                Ok(v) => v,
                Err(_) => {
                    error!("Failed to parse payload: {:?}", p.payload);
                    continue;
                }
            };

            // Fix timestamp logic: thay thế timestamp của payload bằng timestamp hiện tại của server
            // để tránh lỗi watchdog đánh dấu chết node do lệch thời gian
            payload.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            info!("Received MQTT payload: {:?}", payload);

            let payload = Arc::new(payload);

            state.latest_data.insert(payload.node_id.clone(), payload.clone());

            // Process payload: update data, run fire detection, update path if needed
            state.process_payload(&payload);

            tokio::spawn({
                let payload = payload.clone();
                let state = state.clone();
                async move {
                    if let Err(e) = state.db_tx.send((*payload).clone()).await {
                        error!("Failed to send payload to db_tx: {:?}", e);
                    }
                }
            });
        }
    }
}
