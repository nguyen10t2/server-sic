import paho.mqtt.client as mqtt
import json
import time
import random
import os
import heapq

BROKER = "localhost"
PORT = 1883
BASE_TOPIC = "fire/"

GRAPH_FILE = os.path.join(os.path.dirname(__file__), "..", "backend", "building_graph.json")


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


def on_connect(client, userdata, flags, reason_code, properties):
    print("Connected to MQTT broker:", reason_code)


def run_simulation():

    print("=== MENU GIẢ LẬP ESP32 MQTT ===")
    print("1. Kịch bản cháy lan")
    print("2. Kịch bản mất kết nối")
    print("3. Kịch bản báo động giả")

    choice = input("Chọn kịch bản (1-3): ").strip()

    nodes, edges = load_graph()

    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    client.on_connect = on_connect

    try:
        client.connect(BROKER, PORT, 60)
    except Exception:
        print("Không thể kết nối MQTT broker")
        return

    client.loop_start()

    start_time = time.time()

    start_fire_node = 8
    node_distances = dijkstra(nodes, edges, start_fire_node)

    max_dist = max(d for d in node_distances.values() if d != float("inf"))
    if max_dist == 0:
        max_dist = 1

    fire_schedule = {}

    for n in nodes:
        d = node_distances[n]

        if d == float("inf"):
            fire_schedule[n] = 999999
        else:
            fire_schedule[n] = (d / max_dist) * 280

    print("\nBắt đầu giả lập. Ctrl+C để dừng.")

    try:

        while True:

            current_time = time.time()
            elapsed = current_time - start_time

            for node_id in nodes:

                if choice == "2" and node_id == 12:
                    continue

                timestamp = int(current_time * 1000)

                temp = random.uniform(24, 26)
                smoke = random.uniform(10, 50)

                flame = False
                status = 0

                if choice == "1":

                    t_fire = fire_schedule.get(node_id, 9999)

                    if elapsed >= t_fire:

                        temp = random.uniform(80, 100)
                        smoke = random.uniform(300, 600)

                        flame = True
                        status = 2

                    elif elapsed >= t_fire - 30 and elapsed < t_fire:

                        progress = (elapsed - (t_fire - 30)) / 30

                        temp = 26 + progress * 54
                        smoke = 50 + progress * 250

                        status = 1

                elif choice == "3":

                    if random.random() < 0.05:
                        temp = random.uniform(90, 110)
                        smoke = random.uniform(400, 700)
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

                if status == 2:
                    print(f"Node {node_id:2} -> CHÁY!")
                elif status == 1:
                    print(f"Node {node_id:2} -> Cảnh báo")
                else:
                    print(f"Node {node_id:2} -> Bình thường")

                time.sleep(random.uniform(0.5, 0.8))

            print("-" * 40)

            time.sleep(2)

    except KeyboardInterrupt:

        print("\nDừng giả lập")

        client.loop_stop()
        client.disconnect()


if __name__ == "__main__":
    run_simulation()