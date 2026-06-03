# Node Provisioning Specification

## Mục tiêu

Cho phép thêm Node mới vào hệ thống mà không cần sửa firmware.

---

## Provisioning Flow

Admin Upload Building Graph

↓

Backend Parse Building Graph

↓

Backend Generate Node Config

↓

Node Request Config

↓

Config Delivered

↓

Config Saved To Flash

↓

Node Operational

---

## Node Configuration

Ví dụ:

{
  "node_id": "N302",
  "floor": 3,
  "neighbors": [
    "N301",
    "N303",
    "STAIR_3"
  ],
  "static_exit": "STAIR_3"
}

---

## Storage

ESP32 lưu config vào:

- LittleFS
hoặc
- SPIFFS

Ví dụ:

config.json

---

## Startup Sequence

Boot

↓

Load Config

↓

Connect WiFi

↓

Connect MQTT

↓

Send Heartbeat

↓

Ready

---

## Reconfiguration

Khi nhận config mới:

config/{node_id}

Node:

- Validate
- Save Flash
- Reload Config

---

## Fail Cases

### Config Missing

Hiển thị:

CONFIG ERROR

---

### MQTT Unavailable

Sử dụng static_exit.

---

### Corrupted Config

Load backup config gần nhất.