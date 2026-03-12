use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};


/**
 * Struct đại diện cho payload nhận được từ MQTT, có thể mở rộng thêm các trường khác nếu cần
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub node_id: String,    // ID của node gửi dữ liệu
    pub temperature: f32,   // Nhiệt độ
    pub humidity: f32,      // Độ ẩm
    pub smoke: f32,         // Mức độ khói
    pub fire: bool,         // Có cháy hay không
    pub timestamp: i64,     // Thời gian gửi dữ liệu (timestamp)
}

/**
 * Struct lưu trữ state chung của ứng dụng, ở đây là dữ liệu mới nhất từ MQTT
 */
pub struct AppState {
    pub latest_data: Mutex<HashMap<String, Payload>>, // Lưu trữ dữ liệu mới nhất từ mỗi node_id
}