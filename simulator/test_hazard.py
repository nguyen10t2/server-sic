from hazard_map import HazardMap


hazard = HazardMap()

hazard.update(
    "N303",
    100
)

print(
    hazard.get_score(
        "N303"
    )
)