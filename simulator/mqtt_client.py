import json
import paho.mqtt.client as mqtt

from config import MQTT_BROKER, MQTT_PORT


class MQTTClient:

    def __init__(self):
        self.client = mqtt.Client(
            mqtt.CallbackAPIVersion.VERSION2
        )

        self.client.connect(
            MQTT_BROKER,
            MQTT_PORT,
            60
        )

    def publish_sensor(self, payload):

        topic = f"sensor/{payload['node_id']}"

        self.client.publish(
            topic,
            json.dumps(payload),
            qos=1
        )