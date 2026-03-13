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
fn reconstruct_path(previous: &HashMap<u8, u8>, start: u8, end: u8) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::graph::Edge;
    use crate::database::schema::NodeStatus;

    #[test]
    fn test_build_adjacency_list() {
        let graph = Graph {
            nodes: vec![1, 2, 3],
            edges: vec![Edge { from: 1, to: 2, weight: 4.0 }, Edge { from: 2, to: 3, weight: 4.0 }],
        };

        let adj = build_adjacency_list(&graph);

        // Check neighbors of node 1
        assert!(adj.get(&1).unwrap().contains(&(2, 4.0)));
        // Check neighbors of node 2
        assert!(adj.get(&2).unwrap().contains(&(1, 4.0)));
        assert!(adj.get(&2).unwrap().contains(&(3, 4.0)));
        // Check neighbors of node 3
        assert!(adj.get(&3).unwrap().contains(&(2, 4.0)));
    }

    #[test]
    fn test_reconstruct_path() {
        let mut previous = HashMap::new();
        previous.insert(2, 1);
        previous.insert(3, 2);

        let path = reconstruct_path(&previous, 1, 3);

        assert_eq!(path, vec![1, 2, 3]);
    }

    #[test]
    fn test_dijkstra_simple() {
        let graph = Graph {
            nodes: vec![1, 2, 3, 4, 5],
            edges: vec![
                Edge { from: 1, to: 2, weight: 4.0 },
                Edge { from: 2, to: 3, weight: 4.0 },
                Edge { from: 3, to: 4, weight: 4.0 },
                Edge { from: 4, to: 5, weight: 4.0 },
                Edge { from: 1, to: 5, weight: 20.0 },
            ],
        };

        let adj = build_adjacency_list(&graph);
        let latest_data = DashMap::new();

        // Test từ node 1 đến exit 5
        let result = dijkstra(&graph, &adj, 1, &[5], &latest_data);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.exit_node, 5);
        // Với default payload (temp=25, humidity=50), danger = 1.0
        // weight = base_distance * (1 + danger) = 4 * 2 = 8 per edge
        // Path 1->2->3->4->5 = 8*4 = 32
        assert_eq!(result.total_weight, 32.0);
    }

    #[test]
    fn test_dijkstra_with_fire() {
        let graph = Graph {
            nodes: vec![1, 2, 3, 4, 5],
            edges: vec![
                Edge { from: 1, to: 2, weight: 4.0 },
                Edge { from: 2, to: 3, weight: 4.0 },
                Edge { from: 3, to: 4, weight: 4.0 },
                Edge { from: 4, to: 5, weight: 4.0 },
                Edge { from: 1, to: 5, weight: 10.0 },
            ],
        };

        let adj = build_adjacency_list(&graph);
        let latest_data = DashMap::new();

        // Simulate fire at node 2
        let fire_payload = Payload {
            temperature: 100.0,
            humidity: 30.0,
            smoke: 500.0,
            flame: true,
            node_id: 2,
            status: NodeStatus::NODEFIRE as u8,
            ..Default::default()
        };
        latest_data.insert(2, Arc::new(fire_payload));

        let result = dijkstra(&graph, &adj, 1, &[5], &latest_data);

        assert!(result.is_some());
        let result = result.unwrap();
        // Should go through 1->5 directly instead of through node 2
        assert_eq!(result.exit_node, 5);
    }
}
