/// Các hằng số cấu hình cho hệ thống

/// Fire detection thresholds
pub mod fire_detection {
    /// Temperature threshold (°C) - trên ngưỡng này được coi là nguy hiểm
    pub const TEMPERATURE_THRESHOLD: f32 = 60.0;
    pub const TEMP_WARNING_THRESHOLD: f32 = 40.0;
    pub const TEMP_MAX_RANGE: f32 = 20.0;

    /// Smoke threshold (ppm) - trên ngưỡng này được coi là nguy hiểm  
    pub const SMOKE_THRESHOLD: f32 = 750.0;
    pub const SMOKE_WARNING_THRESHOLD: f32 = 700.0;
    pub const SMOKE_MAX_RANGE: f32 = 50.0;

    pub const TREND_TEMP_THRESHOLD: f32 = 20.0;
    pub const TREND_SMOKE_THRESHOLD: f32 = 200.0;

    /// Fire probability threshold (0.0-1.0) - xác nhận có cháy khi vượt ngưỡng
    pub const FIRE_THRESHOLD: f32 = 0.7;

    /// Số lần đọc liên tiếp phải vượt ngưỡng mới confirm (tránh false positive)
    pub const CONFIRMATION_COUNT: usize = 2;

    /// Maximum history size cho mỗi node (số readings giữ lại)
    pub const MAX_HISTORY_SIZE: usize = 20;
}

/// Weight calculation
pub mod weight {
    /// Trọng số cho các yếu tố trong tính fire probability
    pub const WEIGHT_SMOKE: f32 = 0.45;
    pub const WEIGHT_TEMPERATURE: f32 = 0.30;
    pub const WEIGHT_TREND: f32 = 0.15;
    pub const WEIGHT_ANOMALY: f32 = 0.10;
}

/// Danger calculation parameters
pub mod danger {
    pub const TEMP_MAX_SCORE: f32 = 100.0;
    pub const SMOKE_MAX_SCORE: f32 = 500.0;

    pub const TEMP_WEIGHT: f32 = 2.0;
    pub const SMOKE_WEIGHT: f32 = 3.0;
    pub const FLAME_PENALTY: f32 = 10.0;
}

/// Building configuration
pub mod building {
    pub const TOTAL_NODES: u16 = 20;

    /// Default edge weight khi không tìm thấy trong graph
    pub const DEFAULT_EDGE_WEIGHT: f32 = 4.0;
}

/// MQTT configuration
pub mod mqtt {
    /// Topic pattern để subscribe
    pub const TOPIC_PATTERN: &str = "fire/#";

    /// Keep alive interval (seconds)
    pub const KEEP_ALIVE_SECS: u64 = 5;
}

/// Default sensor values (khi không có data)
pub mod defaults {
    pub const TEMPERATURE: f32 = 25.0; // Nhiệt độ phòng
    pub const SMOKE: f32 = 0.0; // Không có khói
    pub const FLAME: bool = false; // Không có lửa
    pub const BATTERY: u8 = 100; // Pin đầy
}
