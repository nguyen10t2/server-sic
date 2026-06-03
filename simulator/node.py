import random


class VirtualNode:

    def __init__(self, node_id):

        self.node_id = node_id

        self.temperature = 30
        self.smoke = 20
        self.flame = False

    def generate_sensor_data(self):

        return {
            "node_id": self.node_id,
            "temperature": round(
                self.temperature + random.uniform(-2, 2),
                2
            ),
            "smoke": round(
                self.smoke + random.uniform(-5, 5),
                2
            ),
            "flame": self.flame
        }