use crate::state::app_state::AppState;
use crate::common::fire_detection::FireDetectionResult;
use actix_web::{HttpResponse, Responder, get, web};

/// Lấy tất cả dữ liệu sensor mới nhất
#[get("/api/status")]
pub async fn get_status(data: web::Data<AppState>) -> impl Responder {
    let current_val =
        data.latest_data.iter().map(|entry| entry.value().clone()).collect::<Vec<_>>();

    HttpResponse::Ok().json(current_val)
}

/// Lấy fire detection status cho tất cả nodes
#[get("/api/fire/status")]
pub async fn get_fire_status(data: web::Data<AppState>) -> impl Responder {
    let fire_status = data.get_all_fire_status();
    let has_fire = data.has_fire();

    #[derive(serde::Serialize)]
    struct FireStatusResponse {
        has_fire: bool,
        fire_nodes: Vec<FireDetectionResult>,
    }

    HttpResponse::Ok().json(FireStatusResponse {
        has_fire,
        fire_nodes: fire_status,
    })
}

/// Lấy evacuation path cho một node cụ thể
#[get("/api/evacuate/{node_id}")]
pub async fn get_evacuation_path(
    data: web::Data<AppState>,
    node_id: web::Path<u16>,
) -> impl Responder {
    let node = *node_id;
    
    if let Some(path) = data.get_evacuation_path(node) {
        #[derive(serde::Serialize)]
        struct EvacuationResponse {
            node_id: u16,
            path: Vec<u8>,
            total_weight: f32,
            exit_node: u8,
            has_fire: bool,
        }

        HttpResponse::Ok().json(EvacuationResponse {
            node_id: node,
            path: path.path,
            total_weight: path.total_weight,
            exit_node: path.exit_node,
            has_fire: data.has_fire(),
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "No evacuation path found",
            "node_id": node,
        }))
    }
}

/// Lấy evacuation paths cho tất cả nodes
#[get("/api/evacuate/all")]
pub async fn get_all_evacuation_paths(data: web::Data<AppState>) -> impl Responder {
    let has_fire = data.has_fire();
    
    // Get all cached paths
    let paths: Vec<_> = data.cached_path.iter()
        .map(|entry| {
            let node_id = entry.key();
            let path = entry.value();
            
            serde_json::json!({
                "node_id": *node_id,
                "path": path.path.clone(),
                "total_weight": path.total_weight,
                "exit_node": path.exit_node,
            })
        })
        .collect();

    #[derive(serde::Serialize)]
    struct EvacuationAllResponse {
        has_fire: bool,
        #[serde(flatten)]
        paths: Vec<serde_json::Value>,
    }

    HttpResponse::Ok().json(EvacuationAllResponse {
        has_fire,
        paths,
    })
}

/// Lấy thông tin building graph
#[get("/api/building/graph")]
pub async fn get_building_graph(data: web::Data<AppState>) -> impl Responder {
    #[derive(serde::Serialize)]
    struct GraphResponse {
        nodes: Vec<u8>,
        edges: Vec<(u8, u8, f32)>,
        exits: Vec<u8>,
    }

    let edges: Vec<_> = data.graph.edges.iter()
        .map(|e| (e.from, e.to, e.weight))
        .collect();

    HttpResponse::Ok().json(GraphResponse {
        nodes: data.graph.nodes.clone(),
        edges,
        exits: vec![5, 10, 15, 20],
    })
}
