# Route Payload Specification

Topic:

route/{node_id}

Example:

route/12

Payload:

{
  "node_id": 12,
  "path": [12,17,18,19,20],
  "next_node": 17,
  "direction": "S",
  "exit_node": 20,
  "version": 1
}

Fields

node_id:
Current node

path:
Full evacuation path

next_node:
Immediate next hop

direction:
LED direction

exit_node:
Target exit

version:
Route version