# 📡 Giao Thức Truyền Thông & Cấu Trúc Payload

Tài liệu này định nghĩa cấu trúc dữ liệu giao tiếp giữa các thiết bị đầu cuối - Node không dây (ESP32), máy chủ trung tâm (Rust Backend) và Web/App Frontend. Cấu trúc này bám sát tệp mã nguồn `schema.rs` của Backend.

---

## 1. Bản Tin Cảm Biến Của Node (ESP32 -> Backend) | Uplink Payload

Gói dữ liệu thu thập từ các phòng được gửi đến Backend thông qua giao thức truyền tải (MQTT/HTTP). Mỗi gói tin báo cáo trạng thái môi trường hiện tại của Node.

**Định dạng JSON dựa trên struct `Payload`:**
```json
{
  "timestamp": 1715423894000, // Unix timestamp từ node (nếu có) hoặc để Backend tự gán
  "node_id": 1,               // ID định danh của node theo sơ đồ toà nhà (u16)
  "temperature": 35.5,        // Cảm biến nhiệt độ (độ C - f32)
  "humidity": 68.2,           // Cảm biến độ ẩm (% - f32)
  "smoke": 420.0,             // Mức phát hiện khói, khí CO/Methan (Analog value/ppm - f32)
  "flame": false,             // Trạng thái phát hiện lửa trực tiếp (true: CÓ LỬA, false: Không)
  "battery": 95,              // Phần trăm pin còn lại (u8)
  "status": 0                 // Trạng thái nút (u8): 0=ALIVE, 1=WARNING, 2=FIRE, 3=DEAD
}
```
*Ghi chú: Giá trị `status` thường do Backend tính toán lại dựa trên hệ tri thức và Timeout liên lạc (Watchdog).*

---

## 2. Lệnh Điều Khiển Xung Đột (Backend -> ESP32) | Downlink Payload

Gói lệnh này được Backend trả về từng ESP32 riêng biệt sau khi thuật toán `Path Finding (Dijkstra)` hoàn thành việc định tuyến thoát hiểm cho hệ thống.

**Định dạng JSON dựa trên struct `CommandPayload`:**
```json
{
  "buzzer": true,  // Kích hoạt còi hú báo động (true = Bật, false = Tắt)
  "dir": "E"       // Lệnh hướng cho LED Matrix ("N", "S", "E", "W", "OFF") - N: Bắc, S: Nam, E: Đông, W: Tây
}
```

*   **Logic thực thi ở vi điều khiển (MCU):** 
    * Nếu nhận được `buzzer: true`, Node sẽ liên tục rú còi SOS cảnh báo không được tới gần.
    * Mảng LED Matrix sẽ được xuất xung để hiển thị ký hiệu mũi tên (N: Lên Bắc, S: Xuống Nam, E: Sang Đông, W: Sang Tây) tương ứng với giá trị `dir` nhằm hướng dẫn người sơ tán. Trạng thái `OFF` nếu đã an toàn hoặc nằm ngoài luồng định tuyến.

---

## 3. Giao Tiếp Lên Frontend Giám Sát (Backend -> Web/App) | WebSocket Payload

Bản tin trung tâm chuyển tiếp qua WebSocket để hiển thị real-time trạng thái của cả khu vực lên Giao diện giám sát.

**Định dạng JSON dựa trên struct `WsMessage`:**
```json
{
  "type": "NODE_UPDATE",          // Loại phân loại bản tin (Ví dụ: "NODE_UPDATE", "EVACUATION_ROUTES")
  "payload": {                    // Bê nguyên struct Payload của Node ở phần 1 vào đây
    "timestamp": 1715423894000,
    "node_id": 1,
    "temperature": 38.0,
    "humidity": 65.0,
    "smoke": 300.0,
    "flame": false,
    "battery": 95,
    "status": 1
  },
  "evacuation_paths": [           // Mảng chứa lộ trình sơ tán, mảng này tuỳ chọn (Option) tùy từng thời điểm
     [1, 2, 3, 5],                // Ví dụ: Node 1 -> 2 -> 3 -> 5
     [12, 11, 15] 
  ]
}
```
