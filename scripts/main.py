import paho.mqtt.client as mqtt
import json
import time
import random
import os
import heapq

BROKER = "127.0.0.1"
PORT = 1883
TOPIC = "esp32/sensors"

# Đọc cấu trúc đồ thị từ file json
GRAPH_FILE = os.path.join(os.path.dirname(__file__), "..", "backend", "building_graph.json")

def load_graph():
    try:
        with open(GRAPH_FILE, "r") as f:
            data = json.load(f)
            return data["nodes"], data["edges"]
    except Exception as e:
        print(f"Không thể đọc building_graph.json: {e}")
        return range(1, 21), []

def dijkstra(nodes, edges, start_node):
    # Adjacency list
    adj = {n: [] for n in nodes}
    for e in edges:
        u, v, w = e
        if len(e) >= 3:
            adj[u].append((v, w))
            adj[v].append((u, w))
    
    dist = {n: float('inf') for n in nodes}
    dist[start_node] = 0
    pq = [(0, start_node)]
    
    while pq:
        d, u = heapq.heappop(pq)
        if d > dist[u]:
            continue
        for v, w in adj[u]:
            if dist[u] + w < dist[v]:
                dist[v] = dist[u] + w
                heapq.heappush(pq, (dist[v], v))
    return dist

def run_simulation():
    print("=== MENU GIẢ LẬP ESP32 MQTT ===")
    print("1. Kịch bản cháy lan (Fire spread - 5 phút lan hết tòa nhà)")
    print("2. Kịch bản mất kết nối (Dead node)")
    print("3. Kịch bản báo động giả / Nhiễu (False alarm)")
    choice = input("Vui lòng chọn kịch bản (1-3): ").strip()
    
    nodes, edges = load_graph()
    
    client = mqtt.Client()
    try:
        client.connect(BROKER, PORT, 60)
    except Exception as e:
        print("Không thể kết nối tới MQTT Broker. Vui lòng kiểm tra lại Broker.")
        return

    start_time = time.time()
    
    # Chuẩn bị dữ liệu cho kịch bản 1
    start_fire_node = 8
    node_distances = dijkstra(nodes, edges, start_fire_node)
    max_dist = max(d for d in node_distances.values() if d != float('inf'))
    if max_dist == 0:
        max_dist = 1
    
    # Tính thời điểm bắt lửa (T_fire) cho từng node
    # Tối đa 5 phút (300s) -> Ta scale max_dist tương ứng với 280 giây
    fire_schedule = {}
    for n in nodes:
        d = node_distances[n]
        if d == float('inf'):
            fire_schedule[n] = 999999
        else:
            fire_schedule[n] = (d / max_dist) * 280.0
            
    print(f"\nBắt đầu chạy giả lập kịch bản {choice}. Nhấn Ctrl+C để dừng.")
    if choice == '1':
        print(f"Lửa bắt đầu từ Node {start_fire_node}, sẽ lan ra toàn bộ trong ~5 phút.")

    try:
        while True:
            current_time = time.time()
            elapsed = current_time - start_time
            
            for node_id in nodes:
                # Bỏ qua node 12 nếu là kịch bản 2
                if choice == '2' and node_id == 12:
                    continue 

                timestamp = int(current_time * 1000)
                
                # Mặc định là bình thường
                temp = random.uniform(24.0, 26.0)
                hum = random.uniform(50.0, 60.0)
                smoke = random.uniform(10.0, 50.0)
                flame = False
                status = 0
                
                if choice == '1':
                    t_fire = fire_schedule.get(node_id, 9999)
                    if elapsed >= t_fire:
                        # Đã cháy hẳn
                        temp = random.uniform(80.0, 100.0)
                        hum = random.uniform(20.0, 30.0)
                        smoke = random.uniform(300.0, 600.0)
                        flame = True
                        status = 2
                    elif elapsed >= t_fire - 30 and elapsed < t_fire:
                        # Giai đoạn nung nóng / cháy từ từ (trước 30s khi cháy hẳn)
                        progress = (elapsed - (t_fire - 30)) / 30.0
                        temp = 26.0 + progress * 54.0 # Tăng từ 26 -> 80
                        hum = 50.0 - progress * 20.0  # Giảm ẩm
                        smoke = 50.0 + progress * 250.0 # Tăng khói
                        flame = False
                        status = 1 # Cảnh báo
                        
                elif choice == '3':
                    # Nhiễu / Báo động giả ngẫu nhiên khoảng 5% xác suất
                    if random.random() < 0.05:
                        temp = random.uniform(90.0, 110.0)
                        smoke = random.uniform(400.0, 700.0)
                        flame = True
                        status = 2

                payload = {
                    "timestamp": timestamp,
                    "temperature": temp,
                    "humidity": hum,
                    "smoke": smoke,
                    "flame": flame,
                    "node_id": node_id,
                    "battery": random.randint(80, 100),
                    "status": status
                }
                
                client.publish(TOPIC, json.dumps(payload))
                
                if choice == '1':
                    state_str = "Bình thường" if status == 0 else ("Cảnh báo" if status == 1 else "CHÁY!")
                    print(f"[{int(elapsed)}s] Node {node_id:>2} | Temp: {temp:5.1f}°C | Trạng thái: {state_str}")
                else:
                    if choice == '3' and status == 2:
                        print(f"Node {node_id:>2} -> CẢNH BÁO GIẢ! Temp: {temp:.1f}°C")
                    else:
                        print(f"Bắn dữ liệu Node {node_id:>2} -> Temp: {temp:.1f}°C")
                
                time.sleep(0.3)
                
            print("-" * 40)
            time.sleep(2.0)

    except KeyboardInterrupt:
        print("\nĐã dừng script giả lập.")
        client.disconnect()

if __name__ == "__main__":
    run_simulation()
