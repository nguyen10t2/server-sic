from graph import BuildingGraph

graph = BuildingGraph("building.json")

print(
    graph.get_neighbors("N302")
)