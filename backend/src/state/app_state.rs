use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::fire_detection::{FireDetectionModel, FireDetectionResult};
use crate::common::graph::Graph;
use crate::common::path_finding::{self, PathResult};
use crate::constants::building::TOTAL_NODES;
use crate::database::schema::{Payload, WsMessage};

/// Struct lưu trữ state chung của ứng dụng
pub struct AppState {
    /// Lưu trữ dữ liệu mới nhất từ mỗi node_id
    pub latest_data: DashMap<u16, Arc<Payload>>,

    /// Mô hình phát hiện cháy
    pub fire_model: FireDetectionModel,

    /// Đồ thị toà nhà
    pub graph: Graph,

    /// Danh sách kề đã được tính toán trước
    pub adjacency_list: HashMap<u8, Vec<(u8, f32)>>,

    /// Lộ trình sơ tán mới nhất (đã được lưu vào bộ nhớ tạm/cache)
    pub cached_path: DashMap<u16, PathResult>,

    /// Kênh broadcast để đẩy dữ liệu tới các kết nối WebSocket realtime
    pub tx: tokio::sync::broadcast::Sender<Arc<WsMessage>>,

    /// Client MQTT để gửi lệnh (command) xuống dưới các node
    pub mqtt_client: Option<rumqttc::AsyncClient>,

    pub db_tx: tokio::sync::mpsc::Sender<Payload>,
}

impl AppState {
    pub fn new(mqtt_client: Option<rumqttc::AsyncClient>, db_tx: tokio::sync::mpsc::Sender<Payload>) -> Self {
        // Tải đồ thị từ tệp JSON
        let graph_json = include_str!("../../building_graph.json");
        let mut graph = Graph { nodes: vec![], edges: vec![] };
        graph.loading_json(graph_json);

        let adjacency_list = path_finding::build_adjacency_list(&graph);

        let (tx, _rx) = tokio::sync::broadcast::channel(100);

        Self {
            latest_data: DashMap::new(),
            fire_model: FireDetectionModel::new(),
            graph,
            adjacency_list,
            cached_path: DashMap::new(),
            tx,
            mqtt_client,
            db_tx,
        }
    }

    /// Xử lý payload từ MQTT: cập nhật dữ liệu, chạy mô hình phát hiện cháy, cập nhật lộ trình sơ tán nếu cần
    pub fn process_payload(&self, payload: &Payload) -> Option<FireDetectionResult> {
        // 1. Cập nhật dữ liệu mới nhất
        self.latest_data.insert(payload.node_id, Arc::new(payload.clone()));

        // 2. Thêm vào mô hình phát hiện cháy
        self.fire_model.add_reading(payload);

        // 3. Phát hiện cháy
        let fire_result = self.fire_model.detect(payload.node_id as u16);

        let mut current_paths_payload = None;

        // 4. Nếu phát hiện có cháy, cập nhật lộ trình sơ tán
        if fire_result.is_fire {
            self.update_evacuation_paths();

            // Extract the paths here to be broadcasted with WS
            let paths: Vec<_> = self.cached_path.iter().map(|entry| {
                serde_json::json!({
                    "node_id": *entry.key(),
                    "path": entry.value().path.clone(),
                    "total_weight": entry.value().total_weight,
                    "exit_node": entry.value().exit_node,
                })
            }).collect();
            current_paths_payload = Some(paths);
        }

        // --- Bắn dữ liệu ra WebSocket ---
        let ws_msg = WsMessage {
            r#type: "SensorAndPathUpdate".to_string(),
            payload: payload.clone(),
            evacuation_paths: current_paths_payload,
        };
        let _ = self.tx.send(Arc::new(ws_msg));

        self.send_command_to_node(payload.node_id);

        Some(fire_result)
    }

    /// Khởi chạy lệnh MQTT (Buzzer + LED) cho một Node
    fn send_command_to_node(&self, node_id: u16) {
        if let Some(client) = self.mqtt_client.clone() {
            let fire_status = self.fire_model.detect(node_id);
            let buzzer = fire_status.is_fire
                || fire_status.risk_level == crate::common::fire_detection::RiskLevel::Critical;

            let mut dir = "OFF".to_string();
            // Lấy bước đi tiếp theo nếu bản thân node không cháy
            if !fire_status.is_fire {
                if let Some(path_result) = self.get_evacuation_path(node_id) {
                    if path_result.path.len() > 1 {
                        dir = path_finding::get_direction(node_id as u8, path_result.path[1]);
                    }
                }
            }

            let topic = format!("esp32/cmd/{}", node_id);
            let cmd = crate::database::schema::CommandPayload { buzzer, dir };

            tokio::spawn(async move {
                if let Ok(json_str) = serde_json::to_string(&cmd) {
                    let _ =
                        client.publish(&topic, rumqttc::QoS::AtLeastOnce, false, json_str).await;
                }
            });
        }
    }

    /// Cập nhật lộ trình sơ tán cho tất cả các node
    fn update_evacuation_paths(&self) {
        let exits = path_finding::default_exits();

        // Lấy tất cả các node đang có cháy
        let fire_nodes = self.fire_model.get_fire_nodes();

        // Với mỗi node bắt đầu tiềm năng (node không có cháy)
        for node_id in 1u16..=TOTAL_NODES {
            if fire_nodes.contains(&node_id) {
                continue;
            }

            // Chạy thuật toán Dijkstra
            if let Some(path) = path_finding::dijkstra(
                &self.graph,
                &self.adjacency_list,
                node_id as u8,
                &exits,
                &self.latest_data,
            ) {
                self.cached_path.insert(node_id, path);
            }
        }
    }

    /// Lấy lộ trình sơ tán cho một node cụ thể
    pub fn get_evacuation_path(&self, node_id: u16) -> Option<PathResult> {
        // Kiểm tra bộ nhớ tạm (cache) trước
        if let Some(path) = self.cached_path.get(&node_id) {
            return Some(path.clone());
        }

        // Nếu không có trong cache, tiến hành tính toán ngay
        let exits = path_finding::default_exits();
        path_finding::dijkstra(
            &self.graph,
            &self.adjacency_list,
            node_id as u8,
            &exits,
            &self.latest_data,
        )
    }

    /// Lấy tất cả kết quả phát hiện cháy
    pub fn get_all_fire_status(&self) -> Vec<FireDetectionResult> {
        self.fire_model.detect_all()
    }

    /// Kiểm tra xem có bất kỳ đám cháy nào trong toà nhà không
    pub fn has_fire(&self) -> bool {
        !self.fire_model.get_fire_nodes().is_empty()
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (db_tx, _) = tokio::sync::mpsc::channel(1);
        Self::new(None, db_tx)
    }
}
