use dashmap::DashMap;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

use crate::common::graph::Graph;
use crate::common::weight;
use crate::constants::{DEFAULT_EXITS, building::DEFAULT_EDGE_WEIGHT};
use crate::database::schema::Payload;

/// Dijkstra result: path từ start đến nearest exit
#[derive(Debug, Clone)]
pub struct PathResult {
    pub path: Vec<u8>,     // Danh sách các node từ start đến exit
    pub total_weight: f32, // Tổng weight của path
    pub exit_node: u8,     // Exit node mà path dẫn đến
}

/// Node trong Dijkstra priority queue
#[derive(Debug, Clone)]
struct DijkstraNode {
    node: u8,
    cost: f32,
}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order vì BinaryHeap là max-heap, ta cần min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DijkstraNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for DijkstraNode {}

/// Default payload cho nodes không có trong latest_data
fn default_payload(node_id: u16) -> Payload {
    Payload { node_id, ..Default::default() }
}

/// Tính weight của edge dựa trên node states
fn edge_weight(graph: &Graph, from: u8, to: u8, latest_data: &DashMap<u16, Arc<Payload>>) -> f32 {
    // Lấy payload cho từng node, hoặc dùng default
    let payload_from = latest_data
        .get(&(from as u16))
        .map(|p| p.value().as_ref().clone())
        .unwrap_or_else(|| default_payload(from as u16));

    let payload_to = latest_data
        .get(&(to as u16))
        .map(|p| p.value().as_ref().clone())
        .unwrap_or_else(|| default_payload(to as u16));

    // Tìm khoảng cách gốc từ graph
    let base_distance = graph
        .edges
        .iter()
        .find(|e| (e.from == from && e.to == to) || (e.from == to && e.to == from))
        .map(|e| e.weight)
        .unwrap_or(DEFAULT_EDGE_WEIGHT); // Default distance nếu không tìm thấy

    // Tính weight động dựa trên danger
    weight(&payload_from, &payload_to, base_distance)
}

/// Build adjacency list từ graph edges
/// Returns: HashMap<node, Vec<(neighbor, base_distance)>>
pub fn build_adjacency_list(graph: &Graph) -> HashMap<u8, Vec<(u8, f32)>> {
    let mut adj: HashMap<u8, Vec<(u8, f32)>> = HashMap::new();

    for edge in &graph.edges {
        // Thêm cả hai chiều (undirected graph cho building evacuation)
        adj.entry(edge.from).or_insert_with(Vec::new).push((edge.to, edge.weight));
        adj.entry(edge.to).or_insert_with(Vec::new).push((edge.from, edge.weight));
    }

    adj
}

/// Dijkstra tìm đường từ start đến nearest exit
///
/// # Arguments
/// * `graph` - Building graph
/// * `adj` - Pre-built adjacency list
/// * `start` - Start node ID
/// * `exits` - List of exit node IDs
/// * `latest_data` - HashMap chứa latest sensor data từ MQTT
///
/// # Returns
/// * `Some(PathResult)` nếu tìm được đường
/// * `None` nếu không có đường nào đến exits
pub fn dijkstra(
    graph: &Graph,
    adj: &HashMap<u8, Vec<(u8, f32)>>,
    start: u8,
    exits: &[u8],
    latest_data: &DashMap<u16, Arc<Payload>>,
) -> Option<PathResult> {
    let mut distances: HashMap<u8, f32> = HashMap::new();
    let mut previous: HashMap<u8, u8> = HashMap::new();
    let mut visited: HashSet<u8> = HashSet::new();
    let mut heap = BinaryHeap::new();

    // Khởi tạo
    distances.insert(start, 0.0);
    heap.push(DijkstraNode { node: start, cost: 0.0 });

    while let Some(DijkstraNode { node, cost }) = heap.pop() {
        // Skip nếu đã visited
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        // Nếu node hiện tại là exit, dừng và reconstruct path
        if exits.contains(&node) {
            let path = reconstruct_path(&previous, start, node);
            return Some(PathResult { path, total_weight: cost, exit_node: node });
        }

        // Lấy neighbors từ adjacency list
        if let Some(neighbors) = adj.get(&node) {
            for &(neighbor, _base_dist) in neighbors {
                if visited.contains(&neighbor) {
                    continue;
                }

                // Tính weight động dựa trên sensor data
                let dynamic_weight = edge_weight(graph, node, neighbor, latest_data);

                // Skip edge nếu weight = vô cùng (node chết hoặc có lửa)
                if dynamic_weight.is_infinite() {
                    continue;
                }

                let new_cost = cost + dynamic_weight;
                let old_cost = distances.get(&neighbor).copied().unwrap_or(f32::INFINITY);

                if new_cost < old_cost {
                    distances.insert(neighbor, new_cost);
                    previous.insert(neighbor, node);
                    heap.push(DijkstraNode { node: neighbor, cost: new_cost });
                }
            }
        }
    }

    // Không tìm được đường đến bất kỳ exit nào
    None
}

/// Reconstruct path từ previous map
pub fn reconstruct_path(previous: &HashMap<u8, u8>, start: u8, end: u8) -> Vec<u8> {
    let mut path = Vec::new();
    let mut current = end;

    while current != start {
        path.push(current);
        match previous.get(&current) {
            Some(&prev) => current = prev,
            None => return vec![start], // Không có đường, chỉ trả về start
        }
    }
    path.push(start);
    path.reverse();
    path
}

/// Default exits cho building (các node ở rìa tòa nhà)
pub fn default_exits() -> Vec<u8> {
    DEFAULT_EXITS.to_vec()
}

/// Tìm đường evacuation với default exits
#[allow(dead_code)]
pub fn find_evacuation_path(
    graph: &Graph,
    adj: &HashMap<u8, Vec<(u8, f32)>>,
    start: u8,
    latest_data: &DashMap<u16, Arc<Payload>>,
) -> Option<PathResult> {
    dijkstra(graph, adj, start, &default_exits(), latest_data)
}

/// Tính toán hướng đi (N, S, E, W) cho con LED matrix theo grid 4x5
pub fn get_direction(from: u8, to: u8) -> String {
    let from_i16 = from as i16;
    let to_i16 = to as i16;

    if to_i16 == from_i16 + 1 {
        "E".to_string()
    } else if to_i16 == from_i16 - 1 {
        "W".to_string()
    } else if to_i16 == from_i16 + 5 {
        "S".to_string() // Số to xuống dưới (Nam)
    } else if to_i16 == from_i16 - 5 {
        "N".to_string() // Số bé lên trên (Bắc)
    } else {
        "OFF".to_string()
    }
}


