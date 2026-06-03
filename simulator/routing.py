import heapq


def shortest_path(graph, start, goal):

    distances = {}

    for node in graph:
        distances[node] = float("inf")

    distances[start] = 0

    previous = {}

    pq = [
        (0, start)
    ]

    while pq:

        current_distance, current_node = (
            heapq.heappop(pq)
        )

        if current_node == goal:
            break

        for neighbor, weight in graph[
            current_node
        ].items():

            distance = (
                current_distance + weight
            )

            if distance < distances[
                neighbor
            ]:

                distances[
                    neighbor
                ] = distance

                previous[
                    neighbor
                ] = current_node

                heapq.heappush(
                    pq,
                    (
                        distance,
                        neighbor
                    )
                )

    path = []

    current = goal

    while current in previous:

        path.append(current)

        current = previous[current]

    path.append(start)

    path.reverse()

    return path