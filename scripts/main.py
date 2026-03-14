import paho.mqtt.client as mqtt
import json
import time
import random
import os
import heapq

BROKER = "localhost"
PORT = 1883
BASE_TOPIC = "fire"

GRAPH_FILE = os.path.join(os.path.dirname(__file__), "..", "backend", "building_graph.json")

# Biến global theo dõi trạng thái cháy từ Node 1
fire_started = False
fire_started_time = None

def load_graph():
    try:
        with open(GRAPH_FILE, "r") as f:
            data = json.load(f)
            return data["nodes"], data["edges"]
    except Exception as e:
        print("Không thể đọc building_graph.json:", e)
        return range(1, 21), []

def dijkstra(nodes, edges, start_node):
    adj = {n: [] for n in nodes}

    for e in edges:
        if len(e) < 3:
            continue
        u, v, w = e
        adj[u].append((v, w))
        adj[v].append((u, w))

    dist = {n: float("inf") for n in nodes}
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

def on_connect(client, userdata, flags, rc, *args):
    print("Connected to MQTT broker:", rc)
    # Lắng nghe dữ liệu từ tất cả các topic fire/# để chắc chắn bắt được Node 1
    client.subscribe(f"{BASE_TOPIC}/#")
    print(f"Đã đăng ký lắng nghe ({BASE_TOPIC}/#)")

def on_message(client, userdata, msg):
    global fire_started, fire_started_time
    if not fire_started:
        try:
            payload = json.loads(msg.payload.decode())
            node_id_val = payload.get("node_id")
            
            # Print nhẹ để debug mọi packet bắt được (loại trừ các giả lập hiện tại)
            if node_id_val in [1, "1"]:
                print(f"[DEBUG] MQTT Node 1 (Topic={msg.topic}): {msg.payload.decode()}")
                # Kiểm tra xem Node 1 có đang bị cháy thật không
                temp = payload.get("temperature", 0)
                flame = payload.get("flame", False)
                status = payload.get("status", 0)
                smoke = payload.get("smoke", 0)
                
                print(f"[DEBUG] Phân tích Node 1: temp={temp}, smoke={smoke}, flame={flame}, status={status}")
                if flame is True or status == 2 or temp > 60 or smoke > 750:
                    print("\n" + "="*50)
                    print("[CẢNH BÁO] Phát hiện Node 1 GẶP CHÁY! Bắt đầu quá trình cháy lan...")
                    print("="*50 + "\n")
                    fire_started = True
                    fire_started_time = time.time()
        except Exception as e:
            # Chỉ hide exception để không spam console, hoặc có thể print nếu cần
            pass

def run_simulation():
    global fire_started, fire_started_time

    print("=== MENU GIẢ LẬP ESP32 MQTT ===")
    print("1. Kịch bản cháy lan (Chờ Node 1 cháy)")
    print("2. Kịch bản mất kết nối")
    print("3. Kịch bản báo động giả")

    choice = input("Chọn kịch bản (1-3): ").strip()

    nodes, edges = load_graph()

    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    client.on_connect = on_connect
    client.on_message = on_message

    try:
        client.connect(BROKER, PORT, 60)
    except Exception:
        print("Không thể kết nối MQTT broker")
        return

    client.loop_start()

    # Bắt đầu cháy lan từ vị trí Node 1
    start_fire_node = 1
    node_distances = dijkstra(nodes, edges, start_fire_node)

    max_dist = max(d for d in node_distances.values() if d != float("inf"))
    if max_dist == 0:
        max_dist = 1

    fire_schedule = {}

    # Giả lập cháy lan dài hơn: 7 phút (420 giây) để cháy hết toàn bộ dựa theo distance
    max_duration = 420  

    for n in nodes:
        d = node_distances[n]

        if d == float("inf"):
            fire_schedule[n] = 999999
        else:
            fire_schedule[n] = (d / max_dist) * max_duration
            
    if choice == "1":
        print("\nBắt đầu giả lập. Đang đợi hơ lửa / bật flame Node 1 để bắt đầu cháy lan... Ctrl+C để dừng.")
    else:
        print("\nBắt đầu giả lập. Ctrl+C để dừng.")
        # Với kịch bản ko phải cháy lan, bỏ qua chờ Node 1
        if choice != "1":
            fire_started = True
            fire_started_time = time.time()

    try:
        while True:
            current_time = time.time()
            if fire_started:
                elapsed = current_time - fire_started_time
            else:
                elapsed = -1

            for node_id in nodes:
                # Bỏ qua Node 1 vì nó là node vật lý đang chạy thật
                if node_id == 1:
                    continue

                if choice == "2" and node_id == 12:
                    continue

                timestamp = int(current_time * 1000)
                temp = random.uniform(24, 26)
                smoke = random.uniform(650, 700)
                flame = False
                status = 0

                if choice == "1":
                    t_fire = fire_schedule.get(node_id, 9999)

                    # Bắt đầu fake số liệu cháy nổ nếu thời gian cháy lan đã tới
                    if elapsed >= t_fire:
                        temp = random.uniform(80, 100)
                        smoke = random.uniform(760, 900)
                        flame = True
                        status = 2
                    # Khoảng 45s trước khi cháy hẳn, nhiệt độ và khói tăng dần
                    elif elapsed >= t_fire - 45 and elapsed >= 0:
                        progress = (elapsed - (t_fire - 45)) / 45
                        temp = 26 + progress * 54
                        smoke = 700 + progress * 60
                        status = 1

                elif choice == "3":
                    if random.random() < 0.05:
                        temp = random.uniform(90, 110)
                        smoke = random.uniform(800, 950)
                        flame = True
                        status = 2

                payload = {
                    "timestamp": timestamp,
                    "node_id": node_id,
                    "temperature": temp,
                    "smoke": smoke,
                    "flame": flame,
                    "battery": random.randint(80, 100),
                    "status": status
                }

                topic = f"{BASE_TOPIC}/{node_id}/sensor"
                result = client.publish(topic, json.dumps(payload), qos=1)

                if result.rc != mqtt.MQTT_ERR_SUCCESS:
                    print("Publish lỗi")

                # Print console log
                if status == 2:
                    print(f"Node {node_id:2} -> CHÁY!")
                elif status == 1:
                    print(f"Node {node_id:2} -> Cảnh báo")
                else:
                    if not fire_started and choice == "1":
                        print(f"Node {node_id:2} -> Bình thường (Chờ báo cháy từ Node 1)")
                    else:
                        print(f"Node {node_id:2} -> Bình thường")

                time.sleep(random.uniform(0.2, 0.3))

            print("-" * 60)
            time.sleep(2)

    except KeyboardInterrupt:
        print("\nDừng giả lập")
        client.loop_stop()
        client.disconnect()

if __name__ == "__main__":
    run_simulation()