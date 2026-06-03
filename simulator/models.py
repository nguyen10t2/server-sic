from dataclasses import dataclass


@dataclass
class NodeState:
    node_id: str
    temperature: float
    smoke: float
    flame: bool