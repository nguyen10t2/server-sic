use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::prelude::FromRow;

/// Enum đại diện cho trạng thái của node, có thể mở rộng thêm các trạng thái khác nếu cần
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
pub enum NodeStatus {
    NODEALIVE = 0,   // Node đang hoạt động bình thường
    NODEWARNING = 1, // Node có cảnh báo nhưng chưa đến mức nguy hiểm
    NODEFIRE = 2,    // Node có lửa
    NODEDEAD = 3,    // Node không hoạt động
}

/// Struct đại diện cho payload nhận được từ MQTT
/// Accepts raw sensor data formats (float for flame, int for status)
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payload {
    pub timestamp: i64,

    pub temperature: f32,
    pub smoke: f32,

    pub node_id: u16,

    pub flame: bool, // Accept bool: true = flame detected, false = no flame
    pub battery: u8,
    pub status: u8, // Accept int: 0=ALIVE, 1=WARNING, 2=FIRE, 3=DEAD
}

impl Payload {
    /// Convert raw payload to NodeStatus enum
    pub fn get_status(&self) -> NodeStatus {
        match self.status {
            0 => NodeStatus::NODEALIVE,
            1 => NodeStatus::NODEWARNING,
            2 => NodeStatus::NODEFIRE,
            3 => NodeStatus::NODEDEAD,
            _ => NodeStatus::NODEALIVE, // Default to ALIVE for unknown values
        }
    }

    /// Check if flame is detected (non-zero)
    pub fn has_flame(&self) -> bool {
        self.flame
    }
}

/// Enum đại diện cho hướng đi
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
pub enum Direction {
    OFF = 0,
    N = 1,
    S = 2,
    E = 3,
    W = 4,
}

/// Enum đại diện cho loại gói tin WebSocket
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WsMessageType {
    SensorAndPathUpdate = 0,
}

/// Struct đại diện cho lệnh gửi xuống các node (Buzzer + LED)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub buzzer: bool,
    pub dir: Direction,
}

/// Dữ liệu WebSocket gửi lên Frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub r#type: WsMessageType,
    pub payload: Payload,
    pub evacuation_paths: Option<Vec<serde_json::Value>>,
}

impl Default for Payload {
    fn default() -> Self {
        use crate::constants::defaults;
        Self {
            timestamp: 0,
            temperature: defaults::TEMPERATURE,
            smoke: defaults::SMOKE,
            flame: defaults::FLAME,
            node_id: 0,
            battery: defaults::BATTERY,
            status: NodeStatus::NODEALIVE as u8,
        }
    }
}
