# Architecture Decision Records

## ADR-001

Decision

Dijkstra chạy trên Backend

Reason

ESP32 chỉ tập trung đọc cảm biến và hiển thị

---

## ADR-002

Decision

MQTT là giao thức chính

Reason

Nhẹ, phù hợp IoT

---

## ADR-003

Decision

Redis cho realtime state

Reason

TTL và truy xuất nhanh

---

## ADR-004

Decision

PostgreSQL cho historical data

Reason

Dễ truy vấn và lưu trữ lâu dài

---

## ADR-005

Decision

Node chỉ lưu neighbors và static_exit

Reason

Giảm bộ nhớ và đơn giản provisioning