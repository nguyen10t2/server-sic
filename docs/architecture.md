# Dynamic Fire Escape Architecture v1

## Overview

Dynamic Fire Escape là hệ thống chỉ dẫn thoát hiểm động sử dụng mạng lưới cảm biến IoT, thuật toán định tuyến thời gian thực và các thiết bị chỉ dẫn trực quan nhằm hướng dẫn người dùng đến lối thoát an toàn nhất trong trường hợp hỏa hoạn.

Khác với biển EXIT truyền thống, hệ thống có khả năng thay đổi hướng dẫn dựa trên diễn biến thực tế của đám cháy.

---

# System Goals

- Phát hiện khu vực nguy hiểm theo thời gian thực.
- Tính toán đường thoát hiểm tối ưu.
- Cập nhật hướng dẫn cho từng Node.
- Hoạt động được ngay cả khi mất kết nối một phần.
- Hỗ trợ giám sát toàn bộ tòa nhà qua Dashboard.

---

# System Components

## ESP32 Nodes

Mỗi Node bao gồm:

### Hardware

- ESP32-S3
- MQ2
- DS18B20
- Flame Sensor
- LED Matrix
- Buzzer
- Backup Battery

### Software

- Sensor Manager
- Network Manager
- Display Manager
- Routing Client
- FailSafe Manager
- Config Manager

---

## Gateway

Thiết bị:

- Raspberry Pi 4 (hoặc tương đương)

Chức năng:

- MQTT Broker
- Route Cache
- Local Processing
- Health Monitoring

Gateway là lớp trung gian giữa các Node và Backend.

---

## Backend

Backend là bộ não của hệ thống.

Bao gồm:

- MQTT Handler
- Fire Analysis Engine
- Hazard Map Generator
- Routing Engine
- WebSocket Gateway
- REST API

---

## Dashboard

Dashboard cung cấp giao diện giám sát.

Hiển thị:

- Building Graph
- Node Status
- Heatmap
- Fire Alerts
- Escape Routes

---

# High Level Architecture

ESP32 Nodes

↓

WiFi

↓

Gateway (Raspberry Pi)

↓

MQTT Broker

↓

Backend

├── Fire Analysis

├── Routing Engine

├── Redis

├── PostgreSQL

├── REST API

└── WebSocket

↓

Dashboard

---

# Building Representation

Toàn bộ tòa nhà được mô hình hóa thành Graph.

## Vertex

Một Node vật lý.

Ví dụ:

- N301
- N302
- N303
- STAIR_3
- EXIT_1

## Edge

Đường di chuyển giữa hai Node.

Ví dụ:

N301 ↔ N302

N302 ↔ STAIR_3

## Weight

Weight được tính động dựa trên:

weight =
distance +
smoke_factor +
temperature_factor +
hazard_factor

Nếu khu vực bị cháy:

weight = infinity

---

# Data Flow

Sensor

↓

ESP32

↓

MQTT

↓

Backend

↓

Fire Analysis

↓

Hazard Map

↓

Routing Engine

↓

Route Update

↓

MQTT

↓

ESP32

↓

LED Matrix

---

# Database Architecture

## Redis

Realtime Data

Lưu:

- Current Node State
- Current Hazard Map
- Route Cache
- Heartbeat Status

TTL mặc định:

30 giây

---

## PostgreSQL

Historical Data

Lưu:

- Sensor History
- Fire Events
- System Logs
- Configuration Versions

---

# Networking

## MQTT

Dùng cho:

- Sensor Data
- Route Updates
- Configuration
- Heartbeat

Topics:

- sensor/{node_id}
- heartbeat/{node_id}
- route/{node_id}
- config/{node_id}
- alert/{node_id}

---

## WebSocket

Dùng cho Dashboard Realtime.

Browser

↓

WebSocket

↓

Backend

---

# Node Configuration

Node KHÔNG lưu toàn bộ Building Graph.

Node chỉ lưu:

{
  "node_id":"N302",
  "neighbors":[
    "N301",
    "N303"
  ],
  "static_exit":"STAIR_3"
}

Điều này giúp giảm bộ nhớ và đơn giản hóa firmware.

---

# Provisioning Flow

Admin Upload Building Graph

↓

Backend Parse Graph

↓

Backend Generate Node Config

↓

Node Request Config

↓

Config Saved To Flash

↓

Node Operational

---

# Fail-Safe Strategy

## Level 1

Backend Failure

Gateway tiếp quản xử lý.

---

## Level 2

Gateway Failure

Node sử dụng Route Cache.

---

## Level 3

Network Isolation

Node sử dụng Static Exit.

---

## Level 4

No Available Data

Hiển thị:

FOLLOW STATIC EXIT

---

# Routing Strategy

Phiên bản hiện tại:

- Dijkstra chạy trên Backend

ESP32 chỉ:

- Gửi dữ liệu cảm biến
- Nhận Route
- Hiển thị chỉ dẫn

Phiên bản tương lai:

- Edge Routing trên ESP32
- Distributed Routing
- Multi-Gateway Support

---

# Simulation System

Mục tiêu:

Cho phép kiểm thử toàn bộ hệ thống mà không cần phần cứng.

Bao gồm:

- Virtual Nodes
- Virtual Sensors
- Virtual Fire
- MQTT Publisher

Kịch bản:

Fire Event

↓

Hazard Map Update

↓

Dijkstra Recalculation

↓

Route Update

↓

Dashboard Update

---

# Sprint 1 Deliverables

Architecture Documents

- architecture.md
- protocol.md
- building_graph.md
- provisioning.md
- simulator.md

Backend

- MQTT Integration
- Redis Integration
- Routing Skeleton

Dashboard

- Heatmap
- Node Status

Firmware

- MQTT Communication
- Sensor Reading
- LED Display

Simulator

- Virtual Nodes
- Fire Events

---

# Long-Term Vision

## Version 1

SCIC Prototype

- Dynamic Routing
- Multi Node
- Dashboard

## Version 2

Smart Building Deployment

- Local Routing
- OTA Updates
- Multi Floor Support

## Version 3

Large Scale Deployment

- Multi Building
- Distributed Routing
- AI-based Fire Prediction