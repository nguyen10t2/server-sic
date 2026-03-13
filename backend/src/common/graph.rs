pub struct Edge {
    pub from: u8,
    pub to: u8,
    pub weight: f32,
}

pub struct Graph {
    pub nodes: Vec<u8>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn loading_json(&mut self, json_data: &str) {
        let parsed: serde_json::Value = serde_json::from_str(json_data).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let edges = parsed["edges"].as_array().unwrap();

        self.nodes = nodes.iter().map(|n| n.as_u64().unwrap() as u8).collect();
        self.edges = edges.iter().map(|e| {
            // Edge có thể là array [from, to, weight] hoặc object {"from": x, "to": y, "weight": z}
            let (from, to, weight) = if e.is_array() {
                let arr = e.as_array().unwrap();
                (
                    arr[0].as_u64().unwrap() as u8,
                    arr[1].as_u64().unwrap() as u8,
                    arr[2].as_f64().unwrap() as f32,
                )
            } else {
                (
                    e["from"].as_u64().unwrap() as u8,
                    e["to"].as_u64().unwrap() as u8,
                    e["weight"].as_f64().unwrap() as f32,
                )
            };
            
            Edge { from, to, weight }
        }).collect();
    }
}
