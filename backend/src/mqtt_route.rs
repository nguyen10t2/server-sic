use rumqttc::QoS;

use crate::database::schema::RoutePayload;

pub async fn publish_route(
    client: &rumqttc::AsyncClient,
    payload: &RoutePayload,
) {

    let topic =
        format!(
            "route/{}",
            payload.node_id
        );

    let json =
        serde_json::to_string(payload)
            .unwrap();

    let _ = client
        .publish(
            topic,
            QoS::AtLeastOnce,
            false,
            json
        )
        .await;
}