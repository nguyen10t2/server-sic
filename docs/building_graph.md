# Dynamic Fire Escape Building Graph Specification

## Overview

Toàn bộ tòa nhà được mô hình hóa thành một graph.

Vertex:

- Node vật lý

Edge:

- Đường di chuyển giữa các node

---

## Node Types

### NORMAL

Node hành lang thông thường.

### STAIR

Node cầu thang.

### EXIT

Node lối thoát cuối cùng.

### DANGER

Node bị đánh dấu nguy hiểm.

---

## Node Schema

{
  "id": "N302",
  "floor": 3,
  "type": "NORMAL",
  "position": {
    "x": 100,
    "y": 200
  }
}

---

## Edge Schema

{
  "from": "N301",
  "to": "N302",
  "distance": 5
}

---

## Building Graph Example

{
  "nodes": [
    {
      "id": "N301",
      "floor": 3,
      "type": "NORMAL"
    },
    {
      "id": "N302",
      "floor": 3,
      "type": "NORMAL"
    },
    {
      "id": "STAIR_3",
      "floor": 3,
      "type": "STAIR"
    }
  ],
  "edges": [
    {
      "from": "N301",
      "to": "N302",
      "distance": 5
    },
    {
      "from": "N302",
      "to": "STAIR_3",
      "distance": 4
    }
  ]
}

---

## Dynamic Weight

weight =
distance +
smoke_factor +
temperature_factor +
hazard_factor

---

## Hazard Rules

Safe:

0 - 30

Warning:

31 - 60

Danger:

61 - 80

Critical:

81 - 100

---

## Blocked Area

Nếu node bị cháy:

weight = infinity

Dijkstra không được đi qua node này.