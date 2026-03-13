use dashmap::DashMap;
use std::collections::VecDeque;

use crate::constants::defaults;
use crate::constants::fire_detection::*;
use crate::constants::weight::*;
use crate::database::schema::Payload;

/// Fire detection result
#[derive(Debug, Clone, serde::Serialize)]
pub struct FireDetectionResult {
    pub node_id: u16,
    pub fire_probability: f32, // 0.0 - 1.0
    pub is_fire: bool,         // True nếu fire_probability > threshold
    pub risk_level: RiskLevel,
    pub details: FireDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FireDetails {
    pub temperature_score: f32,
    pub smoke_score: f32,
    pub humidity_score: f32,
    pub flame_factor: f32,
    pub trend_factor: f32,
    pub anomaly_score: f32,
}

/// Historical data cho một node
#[derive(Clone)]
pub struct NodeHistory {
    pub node_id: u16,
    pub readings: VecDeque<Payload>,
    pub max_size: usize,
}

impl NodeHistory {
    pub fn new(node_id: u16) -> Self {
        Self { node_id, readings: VecDeque::new(), max_size: MAX_HISTORY_SIZE }
    }

    pub fn add(&mut self, payload: Payload) {
        self.readings.push_back(payload);
        if self.readings.len() > self.max_size {
            self.readings.pop_front();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.readings.len()
    }

    /// Lấy reading cũ nhất (baseline)
    pub fn baseline(&self) -> Option<&Payload> {
        self.readings.front()
    }

    /// Lấy reading mới nhất
    pub fn latest(&self) -> Option<&Payload> {
        self.readings.back()
    }

    /// Tính average của một trường trong history
    pub fn avg_temperature(&self) -> f32 {
        if self.readings.is_empty() {
            return defaults::TEMPERATURE; // Default room temperature
        }
        self.readings.iter().map(|p| p.temperature).sum::<f32>() / self.readings.len() as f32
    }

    pub fn avg_smoke(&self) -> f32 {
        if self.readings.is_empty() {
            return defaults::SMOKE;
        }
        self.readings.iter().map(|p| p.smoke).sum::<f32>() / self.readings.len() as f32
    }

    #[allow(dead_code)]
    pub fn avg_humidity(&self) -> f32 {
        if self.readings.is_empty() {
            return defaults::HUMIDITY;
        }
        self.readings.iter().map(|p| p.humidity).sum::<f32>() / self.readings.len() as f32
    }

    /// Tính trend (sự thay đổi từ baseline đến latest)
    pub fn temperature_trend(&self) -> f32 {
        if let (Some(baseline), Some(latest)) = (self.baseline(), self.latest()) {
            return latest.temperature - baseline.temperature;
        }
        0.0
    }

    pub fn smoke_trend(&self) -> f32 {
        if let (Some(baseline), Some(latest)) = (self.baseline(), self.latest()) {
            return latest.smoke - baseline.smoke;
        }
        0.0
    }
}

/// Fire Detection Model
///
/// Sử dụng kết hợp:
/// 1. Rule-based detection (thresholds)
/// 2. Anomaly detection (so với historical baseline)
/// 3. Trend analysis (sự gia tăng đột ngột)
pub struct FireDetectionModel {
    /// Ngưỡng temperature để detect fire
    temperature_threshold: f32,
    /// Ngưỡng smoke để detect fire
    smoke_threshold: f32,
    /// Ngưỡng fire_probability để xác nhận có cháy
    fire_threshold: f32,
    /// Số lần đọc liên tiếp phải vượt ngưỡng mới confirm
    confirmation_count: usize,
    /// Lưu trữ history cho mỗi node
    history: DashMap<u16, NodeHistory>,
    /// Đếm số lần liên tiếp vượt ngưỡng
    consecutive_alerts: DashMap<u16, usize>,
}

impl FireDetectionModel {
    pub fn new() -> Self {
        Self {
            temperature_threshold: TEMPERATURE_THRESHOLD,
            smoke_threshold: SMOKE_THRESHOLD,
            fire_threshold: FIRE_THRESHOLD,
            confirmation_count: CONFIRMATION_COUNT,
            history: DashMap::new(),
            consecutive_alerts: DashMap::new(),
        }
    }

    /// Thêm reading vào history
    pub fn add_reading(&self, payload: &Payload) {
        let mut history = self
            .history
            .entry(payload.node_id)
            .or_insert_with(|| NodeHistory::new(payload.node_id));

        history.add(payload.clone());
    }

    /// Detect fire cho một node
    pub fn detect(&self, node_id: u16) -> FireDetectionResult {
        let history = self.history.get(&node_id);

        // Nếu không có history, return Safe
        if history.is_none() || history.as_ref().map(|h| h.is_empty()).unwrap_or(true) {
            return FireDetectionResult {
                node_id,
                fire_probability: 0.0,
                is_fire: false,
                risk_level: RiskLevel::Safe,
                details: FireDetails {
                    temperature_score: 0.0,
                    smoke_score: 0.0,
                    humidity_score: 0.0,
                    flame_factor: 0.0,
                    trend_factor: 0.0,
                    anomaly_score: 0.0,
                },
            };
        }

        let history = history.unwrap();
        let latest = history.latest().unwrap();

        // 1. Rule-based scores
        let (temperature_score, smoke_score, humidity_score, flame_factor) =
            self.calculate_sensor_scores(latest);

        // 2. Anomaly detection (so với baseline)
        let (anomaly_score, _temp_anomaly, _smoke_anomaly) =
            self.calculate_anomaly(&history, latest);

        // 3. Trend analysis
        let trend_factor = self.calculate_trend(&history);

        // 4. Tính tổng fire probability
        let fire_probability = self.calculate_fire_probability(
            temperature_score,
            smoke_score,
            humidity_score,
            flame_factor,
            anomaly_score,
            trend_factor,
        );

        // 5. Kiểm tra confirmation (tránh false positive)
        let is_fire = self.check_confirmation(node_id, fire_probability);

        // 6. Determine risk level
        let risk_level = self.determine_risk_level(fire_probability, flame_factor, trend_factor);

        FireDetectionResult {
            node_id,
            fire_probability,
            is_fire,
            risk_level,
            details: FireDetails {
                temperature_score,
                smoke_score,
                humidity_score,
                flame_factor,
                trend_factor,
                anomaly_score,
            },
        }
    }

    /// Tính scores từ sensor readings
    fn calculate_sensor_scores(&self, payload: &Payload) -> (f32, f32, f32, f32) {
        // Temperature score (0-1)
        let temperature_score = if payload.temperature >= self.temperature_threshold {
            1.0
        } else if payload.temperature >= TEMP_WARNING_THRESHOLD {
            (payload.temperature - TEMP_WARNING_THRESHOLD) / TEMP_MAX_RANGE
        } else {
            0.0
        };

        // Smoke score (0-1)
        let smoke_score = if payload.smoke >= self.smoke_threshold {
            1.0
        } else if payload.smoke >= SMOKE_WARNING_THRESHOLD {
            (payload.smoke - SMOKE_WARNING_THRESHOLD) / SMOKE_MAX_RANGE
        } else {
            0.0
        };

        // Humidity score - low humidity = higher fire risk (0-1)
        let humidity_score = if payload.humidity <= HUMIDITY_CRITICAL_THRESHOLD {
            1.0
        } else if payload.humidity <= HUMIDITY_WARNING_THRESHOLD {
            (HUMIDITY_WARNING_THRESHOLD - payload.humidity) / HUMIDITY_MAX_RANGE
        } else {
            0.0
        };

        // Flame factor (instant danger) - use has_flame() method
        let flame_factor = if payload.has_flame() { 1.0 } else { 0.0 };

        (temperature_score, smoke_score, humidity_score, flame_factor)
    }

    /// Tính anomaly score so với baseline
    fn calculate_anomaly(&self, history: &NodeHistory, latest: &Payload) -> (f32, f32, f32) {
        let baseline_temp = history.avg_temperature();
        let baseline_smoke = history.avg_smoke();

        // Anomaly = (current - baseline) / baseline
        let temp_anomaly = if baseline_temp > 0.0 {
            (latest.temperature - baseline_temp) / baseline_temp
        } else {
            0.0
        };

        let smoke_anomaly = if baseline_smoke > 0.0 {
            (latest.smoke - baseline_smoke) / baseline_smoke
        } else {
            0.0
        };

        // Normalize anomaly score (0-1)
        let temp_anomaly_score = temp_anomaly.max(0.0).min(1.0);
        let smoke_anomaly_score = smoke_anomaly.max(0.0).min(1.0);

        let anomaly_score = (temp_anomaly_score + smoke_anomaly_score) / 2.0;

        (anomaly_score, temp_anomaly, smoke_anomaly)
    }

    /// Tính trend factor (sự gia tăng đột ngột)
    fn calculate_trend(&self, history: &NodeHistory) -> f32 {
        if history.len() < 2 {
            return 0.0;
        }

        let temp_trend = history.temperature_trend();
        let smoke_trend = history.smoke_trend();

        // Độ đột ngột dựa trên ngưỡng cài đặt
        let temp_trend_score = (temp_trend / TREND_TEMP_THRESHOLD).max(0.0).min(1.0);
        let smoke_trend_score = (smoke_trend / TREND_SMOKE_THRESHOLD).max(0.0).min(1.0);

        (temp_trend_score + smoke_trend_score) / 2.0
    }

    /// Tính tổng fire probability
    fn calculate_fire_probability(
        &self,
        temperature_score: f32,
        smoke_score: f32,
        humidity_score: f32,
        flame_factor: f32,
        anomaly_score: f32,
        trend_factor: f32,
    ) -> f32 {
        // Trọng số cho các yếu tố
        // Flame là yếu tố quyết định ngay lập tức
        if flame_factor > 0.5 {
            return 1.0;
        }

        // Weight: smoke (40%), temperature (25%), trend (15%), anomaly (10%), humidity (10%)
        let weight_smoke = WEIGHT_SMOKE;
        let weight_temp = WEIGHT_TEMPERATURE;
        let weight_trend = WEIGHT_TREND;
        let weight_anomaly = WEIGHT_ANOMALY;
        let weight_humidity = WEIGHT_HUMIDITY;

        let probability = smoke_score * weight_smoke
            + temperature_score * weight_temp
            + trend_factor * weight_trend
            + anomaly_score * weight_anomaly
            + humidity_score * weight_humidity;

        probability.min(1.0).max(0.0)
    }

    /// Kiểm tra confirmation để tránh false positive
    fn check_confirmation(&self, node_id: u16, probability: f32) -> bool {
        if probability >= self.fire_threshold {
            let mut count = self.consecutive_alerts.entry(node_id).or_insert(0);

            *count += 1;

            // Reset nếu probability thấp
            if probability < self.fire_threshold * 0.5 {
                *count = 0;
            }

            *count >= self.confirmation_count
        } else {
            // Reset counter nếu probability thấp
            let mut count = self.consecutive_alerts.entry(node_id).or_insert(0);
            *count = 0;
            false
        }
    }

    /// Xác định risk level
    fn determine_risk_level(
        &self,
        probability: f32,
        flame_factor: f32,
        trend_factor: f32,
    ) -> RiskLevel {
        // Flame = immediate critical
        if flame_factor > 0.5 {
            return RiskLevel::Critical;
        }

        // High trend = rapidly escalating situation
        if trend_factor > 0.7 && probability > 0.5 {
            return RiskLevel::Critical;
        }

        match probability {
            p if p >= 0.8 => RiskLevel::Critical,
            p if p >= 0.6 => RiskLevel::High,
            p if p >= 0.4 => RiskLevel::Medium,
            p if p >= 0.2 => RiskLevel::Low,
            _ => RiskLevel::Safe,
        }
    }

    /// Detect fire cho tất cả nodes
    pub fn detect_all(&self) -> Vec<FireDetectionResult> {
        let node_ids: Vec<u16> = self.history.iter().map(|r| r.node_id).collect();

        node_ids.into_iter().map(|id| self.detect(id)).collect()
    }

    /// Lấy danh sách nodes có cháy
    pub fn get_fire_nodes(&self) -> Vec<u16> {
        self.detect_all().into_iter().filter(|r| r.is_fire).map(|r| r.node_id).collect()
    }

    /// Reset history cho một node
    #[allow(dead_code)]
    pub fn reset_node(&self, node_id: u16) {
        self.history.remove(&node_id);
        self.consecutive_alerts.remove(&node_id);
    }

    /// Clear tất cả history
    #[allow(dead_code)]
    pub fn clear_all(&self) {
        self.history.clear();
        self.consecutive_alerts.clear();
    }
}

impl Default for FireDetectionModel {
    fn default() -> Self {
        Self::new()
    }
}
