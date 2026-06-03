from graph import BuildingGraph
from routing import shortest_path
from hazard_map import HazardMap


building = BuildingGraph(
    "building.json"
)

hazard = HazardMap()

hazard.update(
    "N303",
    100
)

graph = (
    building.build_weighted_graph(
        hazard
    )
)

path = shortest_path(
    graph,
    "N301",
    "STAIR"
)

print(path)