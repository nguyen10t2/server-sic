import random

from danger_score import calculate_danger


class VirtualNode:

    def __init__(
        self,
        node_id,
        fire=False
    ):

        self.node_id = node_id
        self.fire = fire

    def generate_sensor_data(self):

        if self.fire:

            temperature = random.uniform(
                80,
                100
            )

            smoke = random.uniform(
                250,
                400
            )

            flame = True

        else:

            temperature = random.uniform(
                28,
                35
            )

            smoke = random.uniform(
                10,
                40
            )

            flame = False

        danger_score = calculate_danger(
            temperature,
            smoke,
            flame
        )

        return {
            "node_id": self.node_id,
            "temperature": round(
                temperature,
                2
            ),
            "smoke": round(
                smoke,
                2
            ),
            "flame": flame,
            "danger_score": danger_score
        }