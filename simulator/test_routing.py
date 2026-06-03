from graph import BuildingGraph
from routing import shortest_path


building = BuildingGraph(
    "building.json"
)

graph = (
    building.build_adjacency()
)

path = shortest_path(
    graph,
    "N301",
    "STAIR"
)

print(path)