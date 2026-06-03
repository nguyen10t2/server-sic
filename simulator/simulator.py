import json
import time

from node import VirtualNode


def load_nodes():

    with open("building.json") as f:
        data = json.load(f)

    nodes = []

    for node in data["nodes"]:

        is_fire = (
            node["id"] == "N303"
        )

        nodes.append(
            VirtualNode(
                node["id"],
                fire=is_fire
            )
        )

    return nodes


def main():

    print("Simulator started")

    nodes = load_nodes()

    while True:

        print(
            "\n===================="
        )

        for node in nodes:

            payload = (
                node.generate_sensor_data()
            )

            print(payload)

        time.sleep(2)


if __name__ == "__main__":
    main()