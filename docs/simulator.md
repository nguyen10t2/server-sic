# Simulator Design

## Mục tiêu

Tạo môi trường giả lập để test backend và dashboard trước khi có phần cứng thật.

---

## Components

### Virtual Node

Giả lập ESP32 Node.

### Virtual Sensor

Giả lập:

- Temperature
- Smoke
- Flame

### Virtual Fire

Mô phỏng đám cháy lan.

### MQTT Publisher

Publish dữ liệu giống firmware thật.

---

## Scenario 1

Fire at N303

↓

Smoke tăng

↓

Danger score tăng

↓

Hazard Map cập nhật

↓

Dijkstra chạy lại

↓

Route thay đổi

↓

Dashboard cập nhật

---

## Scenario 2

Node mất kết nối

↓

Heartbeat timeout

↓

Node Offline

↓

Dashboard cập nhật

---

## Scenario 3

Exit bị block

↓

Weight = Infinity

↓

Dijkstra tìm đường mới

↓

Route cập nhật

---

## Simulator Output

Publish:

sensor/{node_id}

heartbeat/{node_id}

Giống firmware ESP32 thật.

---

## Sprint 1 Goal

Có thể giả lập:

- 5 Nodes
- 1 Fire Event
- Dynamic Route Update
- Dashboard Realtime Update