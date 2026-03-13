import paho.mqtt.client as mqtt
import json
import time
import random
from collections import defaultdict

# Cấu hình
BROKER = "localhost"
PORT = 1883
TOPIC_PREFIX = "fire"

# Graph adjacency (building layout)
# Node gần nhau sẽ lan cháy
ADJACENCY = {
    1: [2, 6],
    2: [1, 3, 7],
    3: [2, 4, 8],
    4: [3, 5, 9],
    5: [4, 10],
    
    6: [1, 7, 11],
    7: [2, 6, 8, 12],
    8: [3, 7, 9, 13],
    9: [4, 8, 10, 14],
    10: [5, 9, 15],
    
    11: [6, 12, 16],
    12: [7, 11, 13, 17],
    13: [8, 12, 14, 18],
    14: [9, 13, 15, 19],
    15: [10, 14, 20],
    
    16: [11, 17],
    17: [12, 16, 18],
    18: [13, 17, 19],
    19: [14, 18, 20],
    20: [15, 19],
}

# Fire propagation parameters
ALPHA = 0.05   # Temperature influence
BETA = 0.02    # Smoke influence  
GAMMA = 0.5    # Neighbor fire influence

# Thresholds
TEMP_IGNITE = 70.0    # Nhiệt độ bắt đầu cháy
SMOKE_IGNITE = 400.0  # Smoke bắt đầu cháy

class FireSimulator:
    def __init__(self):
        # Node states: {node_id: {temp, smoke, flame, humidity, battery, status}}
        self.nodes = {}
        self.fire_nodes = set()  # Nodes đang cháy
        
        # Initialize all nodes
        for node_id in range(1, 21):
            self.nodes[node_id] = {
                'temperature': 25.0,
                'humidity': 55.0,
                'smoke': 0.0,
                'flame': False,
                'battery': 92,
                'status': 0,  # NODEALIVE
            }
        
        # Start fire at node 8 (middle of building)
        self.start_fire(8)
    
    def start_fire(self, node_id):
        """Bắt đầu cháy tại một node"""
        self.nodes[node_id]['temperature'] = 85.0
        self.nodes[node_id]['smoke'] = 500.0
        self.nodes[node_id]['flame'] = True
        self.nodes[node_id]['status'] = 2  # NODEFIRE
        self.fire_nodes.add(node_id)
        print(f"Fire started at node {node_id}")
    
    def calculate_spread_probability(self, node_id):
        """Tính xác suất cháy lan sang node"""
        if node_id in self.fire_nodes:
            return 0.0
        
        neighbors = ADJACENCY.get(node_id, [])
        
        # Tính neighbor fire influence
        neighbor_fire_count = sum(1 for n in neighbors if n in self.fire_nodes)
        
        # Lấy current values
        node = self.nodes[node_id]
        
        # Spread probability = α * temp + β * smoke + γ * neighbor_fire
        prob = (ALPHA * node['temperature'] + 
                BETA * node['smoke'] / 100 + 
                GAMMA * neighbor_fire_count)
        
        return prob
    
    def update_node(self, node_id):
        """Cập nhật trạng thái node theo fire propagation"""
        node = self.nodes[node_id]
        
        # Nếu đã cháy, tăng dần temperature và smoke
        if node_id in self.fire_nodes:
            node['temperature'] = min(node['temperature'] + random.uniform(0.5, 2.0), 150.0)
            node['smoke'] = min(node['smoke'] + random.uniform(10, 30), 1000.0)
            node['flame'] = True
            node['status'] = 2  # NODEFIRE
            return
        
        # Nếu chưa cháy, kiểm tra xem có bị lan từ neighbor không
        neighbors = ADJACENCY.get(node_id, [])
        neighbor_fire = any(n in self.fire_nodes for n in neighbors)
        
        if neighbor_fire:
            # Gradient: neighbor càng gần, nhiệt độ càng cao
            # Tăng temp và smoke từ neighbor
            node['temperature'] += random.uniform(1.0, 3.0)
            node['smoke'] += random.uniform(5.0, 20.0)
            node['humidity'] = max(node['humidity'] - random.uniform(0.5, 2.0), 10.0)
            
            # Kiểm tra điều kiện bắt cháy
            if (node['temperature'] > TEMP_IGNITE or 
                node['smoke'] > SMOKE_IGNITE):
                self.start_fire(node_id)
                print(f"Fire spread to node {node_id}! Temp: {node['temperature']:.1f}°C, Smoke: {node['smoke']:.1f}ppm")
        
        # Random fluctuation cho nodes không bị ảnh hưởng
        else:
            node['temperature'] += random.uniform(-0.5, 0.5)
            node['temperature'] = max(20.0, min(node['temperature'], 40.0))
    
    def get_payload(self, node_id) -> dict:
        """Lấy payload cho một node"""
        node = self.nodes[node_id]
        
        return {
            "timestamp": int(time.time() * 1000),
            "temperature": round(node['temperature'], 2),
            "humidity": round(node['humidity'], 2),
            "smoke": round(node['smoke'], 2),
            "flame": bool(node['flame']),
            "node_id": node_id,
            "battery": node['battery'],
            "status": node['status']
        }

def simulate_data():
    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    
    try:
        # Kết nối tới Broker
        client.connect(BROKER, PORT, 60)
        print(f"Fire Simulator started. Sending data to {BROKER}...")
        print(f"Fire propagation: α={ALPHA}, β={BETA}, γ={GAMMA}")
        print(f"gnite thresholds: Temp>{TEMP_IGNITE}°C, Smoke>{SMOKE_IGNITE}ppm")
        
        simulator = FireSimulator()
        
        iteration = 0
        while True:
            iteration += 1
            print(f"\n--- Iteration {iteration} ---")
            print(f"Fire nodes: {sorted(simulator.fire_nodes)}")
            
            # Cập nhật tất cả nodes
            for node_id in range(1, 21):
                simulator.update_node(node_id)
            
            # Gửi dữ liệu cho tất cả nodes
            for node_id in range(1, 21):
                payload = simulator.get_payload(node_id)
                topic = f"{TOPIC_PREFIX}/{node_id}/data"
                
                json_payload = json.dumps(payload)
                client.publish(topic, json_payload)
                
                if node_id in simulator.fire_nodes:
                    print(f"Node {node_id}: T={payload['temperature']:.1f}°C, S={payload['smoke']:.1f}ppm")
            
            # Check if all nodes are on fire
            if len(simulator.fire_nodes) >= 15:
                print(f"\nBuilding almost fully on fire! ({len(simulator.fire_nodes)}/20 nodes)")
            
            # Nghỉ 2 giây trước khi update tiếp
            time.sleep(2.0)
            
    except KeyboardInterrupt:
        print("\nStopping Fire Simulator.")
        client.disconnect()
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    simulate_data()
