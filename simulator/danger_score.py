def calculate_danger(
    temperature: float,
    smoke: float,
    flame: bool
) -> int:

    score = 0

    # smoke tối đa 40 điểm
    score += min(40, smoke / 10)

    # temperature tối đa 40 điểm
    score += min(40, temperature / 2)

    # flame
    if flame:
        score += 20

    return int(min(100, score))