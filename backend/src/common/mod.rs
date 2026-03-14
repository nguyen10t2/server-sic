use crate::constants::danger::*;
use crate::database::schema::{NodeStatus, Payload};

pub mod fire_detection;
pub mod graph;
pub mod path_finding;

fn danger(payload: &Payload) -> f32 {
    if payload.get_status() == NodeStatus::NODEDEAD {
        return f32::INFINITY;
    }

    if payload.get_status() == NodeStatus::NODEFIRE {
        return f32::INFINITY;
    }

    let temp_score = payload.temperature / TEMP_MAX_SCORE;
    let smoke_score = payload.smoke / SMOKE_MAX_SCORE;
    let mut danger = temp_score * TEMP_WEIGHT + smoke_score * SMOKE_WEIGHT;

    if payload.has_flame() {
        danger += FLAME_PENALTY;
    }

    danger
}

pub fn weight(node_1: &Payload, node_2: &Payload, dis: f32) -> f32 {
    let d1 = danger(node_1);
    let d2 = danger(node_2);

    if d1.is_infinite() || d2.is_infinite() {
        return f32::INFINITY;
    }

    let danger_avg = (d1 + d2) / 2.0;

    dis * (1.0 + danger_avg) // Tăng trọng số dựa trên mức độ nguy hiểm
}
