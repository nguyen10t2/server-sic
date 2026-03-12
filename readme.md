# ESP32 Backend

Backend Rust nhận dữ liệu từ thiết bị ESP32 qua MQTT và cung cấp REST API.

## Công nghệ

- **Rust** + **Actix-web** — HTTP server
- **rumqttc** — MQTT client
- **Tokio** — async runtime

## Cách hoạt động

1. Backend kết nối tới MQTT broker và subscribe topic `fire/#`
2. Khi ESP32 gửi dữ liệu lên, backend parse JSON và lưu vào state
3. Client có thể truy vấn state qua REST API

## Cài đặt & Chạy

### Yêu cầu

- Rust (stable)
- MQTT broker (ví dụ: Mosquitto)

### Cấu hình

Tạo file `.env` trong thư mục `backend/`:

```env
ip=localhost
port=8080
mqtt_broker=localhost
mqtt_port=1883
```

### Chạy

```bash
cd backend
cargo run
```

## API

### GET `/api/status`

Trả về dữ liệu mới nhất từ tất cả các node ESP32.

**Response:**

```json
{
  "node_01": {
    "node_id": "node_01",
    "...": "..."
  }
}
```
