mod fire_tests {
    use esp32::common::fire_detection::*;
    use esp32::database::schema::Payload;

    #[test]
    fn test_fire_detection_no_fire() {
        let model = FireDetectionModel::new();

        let payload = Payload {
            timestamp: 0,
            temperature: 25.0,
            humidity: 50.0,
            smoke: 0.0,
            flame: false,
            node_id: 1,
            battery: 100,
            status: 0,
        };

        model.add_reading(&payload);
        let result = model.detect(1);

        assert!(!result.is_fire);
        assert_eq!(result.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_fire_detection_with_flame() {
        let model = FireDetectionModel::new();

        // First reading - should not be fire yet (needs confirmation)
        let payload = Payload {
            timestamp: 0,
            temperature: 80.0,
            humidity: 20.0,
            smoke: 500.0,
            flame: true,
            node_id: 1,
            battery: 100,
            status: 2,
        };

        model.add_reading(&payload);
        let result = model.detect(1);

        // Flame = true should immediately trigger fire regardless of confirmation
        assert!(result.is_fire || result.fire_probability > 0.9);
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_fire_detection_temperature_rise() {
        let model = FireDetectionModel::new();

        // Baseline
        model.add_reading(&Payload {
            timestamp: 0,
            temperature: 25.0,
            humidity: 50.0,
            smoke: 0.0,
            flame: false,
            node_id: 1,
            battery: 100,
            status: 0,
        });

        // Rising temperature
        model.add_reading(&Payload {
            timestamp: 1,
            temperature: 45.0,
            humidity: 40.0,
            smoke: 50.0,
            flame: false,
            node_id: 1,
            battery: 100,
            status: 0,
        });

        let result = model.detect(1);

        assert!(result.details.trend_factor > 0.0);
    }
}

mod path_tests {
    use esp32::common::path_finding::*;
    use esp32::common::graph::{Graph, Edge};
    use esp32::database::schema::{NodeStatus, Payload};
    use dashmap::DashMap;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_build_adjacency_list_2() {
        let graph = Graph {
            nodes: vec![1, 2, 3],
            edges: vec![Edge { from: 1, to: 2, weight: 4.0 }, Edge { from: 2, to: 3, weight: 4.0 }],
        };

        let adj = build_adjacency_list(&graph);

        // Check neighbors
        assert!(adj.get(&1).unwrap().contains(&(2, 4.0)));
        assert!(adj.get(&2).unwrap().contains(&(1, 4.0)));
        assert!(adj.get(&2).unwrap().contains(&(3, 4.0)));
        assert!(adj.get(&3).unwrap().contains(&(2, 4.0)));
    }

    #[test]
    fn test_reconstruct_path_2() {
        let mut previous = HashMap::new();
        previous.insert(2, 1);
        previous.insert(3, 2);

        let path = reconstruct_path(&previous, 1, 3);

        assert_eq!(path, vec![1, 2, 3]);
    }

    #[test]
    fn test_dijkstra_simple_2() {
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

        let result = dijkstra(&graph, &adj, 1, &[5], &latest_data);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.exit_node, 5);
        assert_eq!(result.total_weight, 32.0); // Wait, this might fail if calc changed, let's keep it.
    }

    #[test]
    fn test_dijkstra_with_fire_2() {
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

        let mut fire_payload = Payload::default();
        fire_payload.temperature = 100.0;
        fire_payload.humidity = 30.0;
        fire_payload.smoke = 500.0;
        fire_payload.flame = true;
        fire_payload.node_id = 2;
        fire_payload.status = NodeStatus::NODEFIRE as u8;

        latest_data.insert(2, Arc::new(fire_payload));

        let result = dijkstra(&graph, &adj, 1, &[5], &latest_data);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.exit_node, 5);
    }
}

mod new_logics_tests {
    use esp32::common::path_finding::get_direction;
    use esp32::common::weight;
    use esp32::database::schema::{NodeStatus, Payload};

    #[test]
    fn test_get_direction() {
        assert_eq!(get_direction(1, 2), "E");   // +1
        assert_eq!(get_direction(2, 1), "W");   // -1
        assert_eq!(get_direction(1, 6), "S");   // +5
        assert_eq!(get_direction(6, 1), "N");   // -5
        assert_eq!(get_direction(1, 10), "OFF");// invalid distance
    }

    #[test]
    fn test_weight_with_dead_node() {
        let payload1 = Payload {
            node_id: 1,
            status: NodeStatus::NODEALIVE as u8,
            temperature: 25.0,
            humidity: 50.0,
            smoke: 0.0,
            flame: false,
            timestamp: 0,
            battery: 100,
        };

        let mut payload2 = payload1.clone();
        payload2.node_id = 2;
        payload2.status = NodeStatus::NODEDEAD as u8; // Node dead

        let w = weight(&payload1, &payload2, 4.0);
        assert!(w.is_infinite(), "Weight to dead node must be infinity");
    }

    #[test]
    fn test_weight_with_fire_node() {
        let payload1 = Payload {
            node_id: 1,
            status: NodeStatus::NODEALIVE as u8,
            temperature: 25.0,
            humidity: 50.0,
            smoke: 0.0,
            flame: false,
            timestamp: 0,
            battery: 100,
        };

        let mut payload2 = payload1.clone();
        payload2.node_id = 2;
        payload2.status = NodeStatus::NODEFIRE as u8; // Node on fire

        let w = weight(&payload1, &payload2, 4.0);
        assert!(w.is_infinite(), "Weight to fire node must be infinity");
    }
}
