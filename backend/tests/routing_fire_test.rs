use dashmap::DashMap;
use std::sync::Arc;

use esp32::common::graph::{Edge, Graph};
use esp32::common::path_finding;
use esp32::database::schema::{NodeStatus, Payload};

#[test]
fn test_fire_node_is_avoided() {

    let graph = Graph {
        nodes: vec![1, 2, 3, 4],
        edges: vec![
            Edge {
                from: 1,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 4,
                weight: 2.0,
            },
            Edge {
                from: 4,
                to: 3,
                weight: 2.0,
            },
        ],
        exits: vec![3],
    };

    let adj =
        path_finding::build_adjacency_list(&graph);

    let latest_data =
        DashMap::<u16, Arc<Payload>>::new();

    latest_data.insert(
        2,
        Arc::new(Payload {
            node_id: 2,
            status: NodeStatus::NODEFIRE as u8,
            ..Default::default()
        }),
    );

    let result =
        path_finding::dijkstra(
            &graph,
            &adj,
            1,
            &graph.exits,
            &latest_data,
        )
        .unwrap();

    assert_eq!(
        result.path,
        vec![1, 4, 3]
    );
}