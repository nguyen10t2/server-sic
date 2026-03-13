import paho.mqtt.client as mqtt
import json
import time
import random

BROKER = "127.0.0.1"
PORT = 1883
TOPIC = "esp32/sensors"

client = mqtt.Client()
client.connect(BROKER, PORT, 60)

def generate_payload(node_id):
    # Lấy thời gian thực (millisecond)
    timestamp = int(time.time() * 1000)
    
    # Giả lập 1 node (Ví dụ node 8) đang bị cháy
    is_fire = (node_id == 8)
    
    return {
        "timestamp": timestamp,
        "temperature": random.uniform(80.0, 100.0) if is_fire else random.uniform(24.0, 26.0),
        "humidity": random.uniform(30.0, 40.0) if is_fire else random.uniform(50.0, 60.0),
        "smoke": random.uniform(300.0, 600.0) if is_fire else 0.0,
        "flame": True if is_fire else False,
        "node_id": node_id,
        "battery": random.randint(80, 100),
        "status": 2 if is_fire else 0
    }

try:
    print("🚀 Bắt đầu gửi dữ liệu MQTT giả lập (Nhấn Ctrl+C để dừng)...")
    while True:
        # Gửi tuần tự cho 20 node (Nhưng bỏ qua node 12 để test Watchdog Dead Node)
        for node_id in range(1, 21):
            if node_id == 12:
                continue # Giả lập node 12 bị chết/mất kết nối!
                
            payload = generate_payload(node_id)
            client.publish(TOPIC, json.dumps(payload))
            print(f"Bắn dữ liệu Node {node_id} -> {payload['temperature']:.1f}°C")
            
            # Làm chậm lại 0.3s giữa các node
            time.sleep(0.3) 
            
        print("⏳ Đã quét xong 1 vòng. Chờ chu kỳ tiếp theo...")
        time.sleep(2.0) # Nghỉ 2 giây trước vòng lặp mới

except KeyboardInterrupt:
    print("\n🛑 Đã dừng script giả lập.")
    client.disconnect()
