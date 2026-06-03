import json
import time

from node import VirtualNode


def load_nodes():

    with open("building.json") as f:
        data = json.load(f)

    return [
        VirtualNode(node["id"])
        for node in data["nodes"]
    ]


def main():

    print("Simulator started")

    nodes = load_nodes()

    print("Loaded nodes:", len(nodes))

    while True:

        for node in nodes:

            payload = node.generate_sensor_data()

            print(payload)

        time.sleep(2)


if __name__ == "__main__":
    main()