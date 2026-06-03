import json


class BuildingGraph:

    def __init__(self, path):

        with open(path, "r") as f:
            data = json.load(f)

        self.nodes = data["nodes"]
        self.edges = data["edges"]

    def get_node(self, node_id):

        for node in self.nodes:

            if node["id"] == node_id:
                return node

        return None

    def get_neighbors(self, node_id):

        neighbors = []

        for edge in self.edges:

            if edge["from"] == node_id:
                neighbors.append(edge["to"])

            elif edge["to"] == node_id:
                neighbors.append(edge["from"])

        return neighbors