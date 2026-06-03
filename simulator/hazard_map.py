class HazardMap:

    def __init__(self):

        self.danger = {}

    def update(
        self,
        node_id,
        score
    ):

        self.danger[node_id] = score

    def get_score(
        self,
        node_id
    ):

        return self.danger.get(
            node_id,
            0
        )