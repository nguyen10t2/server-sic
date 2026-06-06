# Dynamic Fire Escape Protocol Specification v1

## MQTT Topics

### Sensor Data

Topic:

sensor/{node_id}

Ví dụ:

sensor/N302

Payload:

{
  "node_id": "N302",
  "temperature": 35.5,
  "smoke": 120,
  "flame": false,
  "timestamp": 1730000000
}

---

### Heartbeat

Topic:

heartbeat/{node_id}

Payload:

{
  "node_id": "N302",
  "status": "online",
  "uptime": 12345
}

---

### Route Update

Topic:

route/{node_id}

Payload:

{
  "node_id": "N302",
  "next_node": "STAIR_3",
  "direction": "LEFT",
  "version": 1
}

Direction:

- LEFT
- RIGHT
- FORWARD
- BACK
- STOP

---

### Configuration Update

Topic:

config/{node_id}

Payload:

{
  "node_id": "N302",
  "neighbors": [
    "N301",
    "N303"
  ],
  "static_exit": "STAIR_3"
}

---

### Alert

Topic:

alert/{node_id}

Payload:

{
  "level": "HIGH",
  "message": "Fire detected"
}

---

## Communication Rules

Sensor Publish Rate:

- 1 lần / 2 giây

Heartbeat:

- 1 lần / 5 giây

MQTT QoS:

- QoS 1

Route Version:

Mỗi lần route thay đổi phải tăng version để tránh dữ liệu cũ ghi đè dữ liệu mới.