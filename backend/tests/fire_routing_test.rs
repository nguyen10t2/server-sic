use esp32::common::graph::{Edge, Graph};
use esp32::common::path_finding::{
    build_adjacency_list,
    dijkstra,
};

use dashmap::DashMap;
use std::sync::Arc;

use esp32::database::schema::Payload;

#[test]
fn test_normal_route_to_exit() {

    let graph = Graph {
        nodes: vec![1, 2, 3],
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
        ],
        exits: vec![3],
    };

    let adj =
        build_adjacency_list(&graph);

    let latest_data:
        DashMap<u16, Arc<Payload>>
        = DashMap::new();

    let result = dijkstra(
        &graph,
        &adj,
        1,
        &[3],
        &latest_data,
    );

    assert!(
        result.is_some()
    );

    let path =
        result.unwrap();

    assert_eq!(
        path.path,
        vec![1, 2, 3]
    );
}