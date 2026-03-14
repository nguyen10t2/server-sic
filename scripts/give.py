import paho.mqtt.client as mqtt

BROKER = "localhost"
PORT = 1883
TOPIC = "esp32/cmd/#"

def on_connect(client, userdata, flags, reason_code, properties):
    print(f"Connected with result code {reason_code}")
    client.subscribe(TOPIC)
    print(f"Subscribed to topic: {TOPIC}")

def on_message(client, userdata, msg):
    print(f"\n[RECEIVED] Topic: {msg.topic}")
    print(f"Message: {msg.payload.decode('utf-8')}")

if __name__ == "__main__":
    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    client.on_connect = on_connect
    client.on_message = on_message
    
    print(f"Connecting to MQTT broker at {BROKER}:{PORT}...")
    client.connect(BROKER, PORT, 60)
    
    # Bắt đầu event loop block thread để lắng nghe message liên tục
    try:
        client.loop_forever()
    except KeyboardInterrupt:
        print("\nDisconnecting...")
        client.disconnect()    
        